//! App state and event loop — core TUI runtime

use crate::screens::{
    engine_picker::EnginePicker,
    history::HistoryBrowser,
    onboarding::OnboardingWizard,
    profile_editor::ProfileEditor,
    result_display::ResultDisplay,
    welcome::WelcomeScreen,
    workflow_picker::WorkflowPicker,
};
use crate::widgets::help::HelpOverlay;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use noesis_sdk::{Config, LocalProfile, NoesisClient};
use ratatui::prelude::*;
use std::time::Duration;
use tracing::{debug, error, info};

/// Which screen is currently active
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveScreen {
    Welcome,
    Onboarding,
    EnginePicker,
    WorkflowPicker,
    ResultDisplay,
    History,
    ProfileEditor,
}

/// Navigation action returned by screens
#[derive(Debug)]
pub enum Action {
    /// Stay on current screen
    None,
    /// Navigate to a different screen
    Navigate(ActiveScreen),
    /// Navigate to result screen with engine output
    ShowEngineResult {
        engine_id: String,
        output: noesis_sdk::EngineOutput,
    },
    /// Navigate to result screen with workflow output
    ShowWorkflowResult {
        workflow_id: String,
        result: noesis_sdk::WorkflowResult,
    },
    /// Quit the application
    Quit,
}

/// Application state
pub struct App {
    /// Which screen is showing
    pub active_screen: ActiveScreen,
    /// SDK config
    pub config: Config,
    /// API client (None if no API key configured)
    pub client: Option<NoesisClient>,
    /// User profile (None if not yet created)
    pub profile: Option<LocalProfile>,
    /// Whether help overlay is visible
    pub show_help: bool,
    /// Global error message (shown briefly)
    pub error_message: Option<String>,
    /// Error display countdown (ticks remaining)
    pub error_ticks: u8,

    // Screen states
    pub welcome: WelcomeScreen,
    pub onboarding: OnboardingWizard,
    pub engine_picker: EnginePicker,
    pub workflow_picker: WorkflowPicker,
    pub result_display: ResultDisplay,
    pub history: HistoryBrowser,
    pub profile_editor: ProfileEditor,
    pub help: HelpOverlay,
}

impl App {
    /// Create a new app, loading config and profile from disk.
    pub async fn new() -> Self {
        let config = Config::load().unwrap_or_default();

        // Try loading API key from keychain
        let api_key: Option<String> = noesis_sdk::KeychainStore::new()
            .get_api_key()
            .ok()
            .flatten();

        // Build config with keychain key if present
        let config = if api_key.is_some() && config.api_key.is_none() {
            Config {
                api_key: api_key.clone(),
                ..config
            }
        } else {
            config
        };

        let client = NoesisClient::new(&config).ok();
        let profile = LocalProfile::load_or_default().ok().flatten();

        // Determine starting screen
        let active_screen = if profile.is_none() {
            ActiveScreen::Onboarding
        } else {
            ActiveScreen::Welcome
        };

        info!(
            "App initialized: profile={}, client={}, screen={:?}",
            profile.is_some(),
            client.is_some(),
            active_screen,
        );

        Self {
            active_screen,
            config,
            client,
            profile,
            show_help: false,
            error_message: None,
            error_ticks: 0,
            welcome: WelcomeScreen::new(),
            onboarding: OnboardingWizard::new(),
            engine_picker: EnginePicker::new(),
            workflow_picker: WorkflowPicker::new(),
            result_display: ResultDisplay::new(),
            history: HistoryBrowser::new(),
            profile_editor: ProfileEditor::new(),
            help: HelpOverlay::new(),
        }
    }

    /// Main event loop.
    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        loop {
            // Draw
            terminal.draw(|frame| self.draw(frame))?;

            // Poll for events with 100ms timeout
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // Global keybinds
                    if self.handle_global_key(key) {
                        break;
                    }

                    // Delegate to active screen
                    let action = self.handle_screen_key(key).await;
                    self.handle_action(action).await;
                }
            }

            // Tick down error display
            if self.error_ticks > 0 {
                self.error_ticks -= 1;
                if self.error_ticks == 0 {
                    self.error_message = None;
                }
            }
        }

        Ok(())
    }

    /// Draw the current screen and overlays.
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Draw active screen
        match self.active_screen {
            ActiveScreen::Welcome => self.welcome.draw(frame, area, &self.profile),
            ActiveScreen::Onboarding => self.onboarding.draw(frame, area),
            ActiveScreen::EnginePicker => self.engine_picker.draw(frame, area),
            ActiveScreen::WorkflowPicker => self.workflow_picker.draw(frame, area),
            ActiveScreen::ResultDisplay => self.result_display.draw(frame, area),
            ActiveScreen::History => self.history.draw(frame, area),
            ActiveScreen::ProfileEditor => self.profile_editor.draw(frame, area, &self.profile),
        }

        // Draw error bar if present
        if let Some(ref msg) = self.error_message {
            crate::widgets::error_bar::draw_error_bar(frame, area, msg);
        }

        // Draw help overlay on top
        if self.show_help {
            self.help.draw(frame, area);
        }
    }

    /// Handle global keybinds. Returns true if app should quit.
    fn handle_global_key(&mut self, key: KeyEvent) -> bool {
        // Ctrl+C / Ctrl+Q → quit
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('q'))
        {
            return true;
        }

        // ? → toggle help (except during onboarding text input)
        if key.code == KeyCode::Char('?') && self.active_screen != ActiveScreen::Onboarding {
            self.show_help = !self.show_help;
        }

        // Esc → close help if open
        if key.code == KeyCode::Esc && self.show_help {
            self.show_help = false;
        }

        false
    }

    /// Delegate key event to the active screen.
    async fn handle_screen_key(&mut self, key: KeyEvent) -> Action {
        if self.show_help {
            return Action::None;
        }

        match self.active_screen {
            ActiveScreen::Welcome => self.welcome.handle_key(key),
            ActiveScreen::Onboarding => {
                self.onboarding.handle_key(key, &mut self.config)
            }
            ActiveScreen::EnginePicker => {
                self.engine_picker
                    .handle_key(key, &self.client, &self.profile)
                    .await
            }
            ActiveScreen::WorkflowPicker => {
                self.workflow_picker
                    .handle_key(key, &self.client, &self.profile)
                    .await
            }
            ActiveScreen::ResultDisplay => self.result_display.handle_key(key),
            ActiveScreen::History => {
                self.history.handle_key(key, &self.client).await
            }
            ActiveScreen::ProfileEditor => {
                self.profile_editor
                    .handle_key(key, &mut self.profile)
            }
        }
    }

    /// Process a navigation action.
    async fn handle_action(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::Quit => {
                // handled in run loop via global keys
            }
            Action::Navigate(screen) => {
                debug!("Navigating to {:?}", screen);

                // On completing onboarding, save profile + keychain + rebuild client
                if self.active_screen == ActiveScreen::Onboarding
                    && screen != ActiveScreen::Onboarding
                {
                    self.finish_onboarding().await;
                }

                self.active_screen = screen;
            }
            Action::ShowEngineResult { engine_id, output } => {
                self.result_display.set_engine_result(engine_id, output);
                self.active_screen = ActiveScreen::ResultDisplay;
            }
            Action::ShowWorkflowResult {
                workflow_id,
                result,
            } => {
                self.result_display.set_workflow_result(workflow_id, result);
                self.active_screen = ActiveScreen::ResultDisplay;
            }
        }
    }

    /// Finalize onboarding: save profile, store API key, rebuild client.
    async fn finish_onboarding(&mut self) {
        // Build profile from wizard data
        if let Some(mut profile) = self.onboarding.build_profile() {
            if let Err(e) = profile.save() {
                self.show_error(format!("Failed to save profile: {e}"));
            } else {
                info!("Profile saved successfully");
                self.profile = Some(profile);
            }
        }

        // Store API key if provided
        let api_key_to_store = self
            .onboarding
            .api_key_input
            .as_ref()
            .filter(|k| !k.is_empty())
            .cloned();

        if let Some(key) = api_key_to_store {
            let store = noesis_sdk::KeychainStore::new();
            if let Err(e) = store.store_api_key(&key) {
                self.show_error(format!("Failed to store API key: {e}"));
            }
            self.config.api_key = Some(key);
        }

        // Rebuild client with new config
        match NoesisClient::new(&self.config) {
            Ok(c) => self.client = Some(c),
            Err(e) => {
                error!("Failed to create API client: {e}");
                self.show_error(format!("Client error: {e}"));
            }
        }
    }

    /// Show an error message for a few seconds.
    fn show_error(&mut self, msg: String) {
        error!("{}", msg);
        self.error_message = Some(msg);
        self.error_ticks = 30; // ~3 seconds at 100ms poll
    }
}
