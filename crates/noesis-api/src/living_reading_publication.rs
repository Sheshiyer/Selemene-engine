use noesis_data::models::living_reading::LivingReadingPublicationArtifactCandidate;
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

pub const ARTIFACT_ROOT_ENV: &str = "LIVING_READING_ARTIFACT_ROOT";
pub const MAX_BYTES_ENV: &str = "LIVING_READING_PUBLICATION_MAX_BYTES";
const DEFAULT_MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_CONFIGURED_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedLivingReadingPublication {
    pub media_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationResolutionError {
    ConfigurationUnavailable,
    UnsupportedProvider,
    UnsupportedMediaType,
    InvalidLocator,
    OutsideArchiveRoot,
    NotRegularFile,
    SizeMismatch,
    SizeLimitExceeded,
    InvalidUtf8,
    InvalidChecksum,
    ChecksumMismatch,
    ReadFailed,
}

impl std::fmt::Display for PublicationResolutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::ConfigurationUnavailable => "publication resolver is not configured",
            Self::UnsupportedProvider => "publication provider is unsupported",
            Self::UnsupportedMediaType => "publication media type is unsupported",
            Self::InvalidLocator => "publication locator is invalid",
            Self::OutsideArchiveRoot => "publication is outside the archive root",
            Self::NotRegularFile => "publication is not a regular file",
            Self::SizeMismatch => "publication size does not match archive metadata",
            Self::SizeLimitExceeded => "publication exceeds the configured size limit",
            Self::InvalidUtf8 => "publication is not valid UTF-8",
            Self::InvalidChecksum => "publication checksum is invalid",
            Self::ChecksumMismatch => "publication checksum does not match archive metadata",
            Self::ReadFailed => "publication could not be read",
        })
    }
}

#[derive(Debug, Clone)]
pub struct LivingReadingPublicationResolver {
    archive_root: PathBuf,
    max_bytes: usize,
}

impl LivingReadingPublicationResolver {
    pub fn from_env() -> Result<Self, PublicationResolutionError> {
        let root = env::var_os(ARTIFACT_ROOT_ENV)
            .filter(|value| !value.is_empty())
            .ok_or(PublicationResolutionError::ConfigurationUnavailable)?;
        let max_bytes = match env::var(MAX_BYTES_ENV) {
            Ok(raw) => raw
                .parse::<usize>()
                .ok()
                .filter(|value| (1..=MAX_CONFIGURED_BYTES).contains(value))
                .ok_or(PublicationResolutionError::ConfigurationUnavailable)?,
            Err(env::VarError::NotPresent) => DEFAULT_MAX_BYTES,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(PublicationResolutionError::ConfigurationUnavailable)
            }
        };
        Self::new(PathBuf::from(root), max_bytes)
    }

    pub fn new(
        archive_root: PathBuf,
        max_bytes: usize,
    ) -> Result<Self, PublicationResolutionError> {
        if !(1..=MAX_CONFIGURED_BYTES).contains(&max_bytes) {
            return Err(PublicationResolutionError::ConfigurationUnavailable);
        }
        let archive_root = fs::canonicalize(archive_root)
            .map_err(|_| PublicationResolutionError::ConfigurationUnavailable)?;
        if !archive_root
            .metadata()
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false)
        {
            return Err(PublicationResolutionError::ConfigurationUnavailable);
        }
        Ok(Self {
            archive_root,
            max_bytes,
        })
    }

    pub fn resolve(
        &self,
        candidate: &LivingReadingPublicationArtifactCandidate,
    ) -> Result<VerifiedLivingReadingPublication, PublicationResolutionError> {
        if candidate.storage_provider != "filesystem" {
            return Err(PublicationResolutionError::UnsupportedProvider);
        }
        let media_type = normalize_media_type(candidate.media_type.as_deref())?;
        let expected_checksum = validate_checksum(&candidate.content_sha256)?;
        let locator = locator_path(&candidate.object_locator)?;
        let target = if locator.is_absolute() {
            locator
        } else {
            self.archive_root.join(locator)
        };
        let canonical_target =
            fs::canonicalize(target).map_err(|_| PublicationResolutionError::ReadFailed)?;
        if !canonical_target.starts_with(&self.archive_root)
            || canonical_target == self.archive_root
        {
            return Err(PublicationResolutionError::OutsideArchiveRoot);
        }

        let file =
            File::open(&canonical_target).map_err(|_| PublicationResolutionError::ReadFailed)?;
        let metadata = file
            .metadata()
            .map_err(|_| PublicationResolutionError::ReadFailed)?;
        if !metadata.is_file() {
            return Err(PublicationResolutionError::NotRegularFile);
        }
        let actual_size = metadata.len();
        if actual_size > self.max_bytes as u64 {
            return Err(PublicationResolutionError::SizeLimitExceeded);
        }
        if candidate.byte_size < 0 || candidate.byte_size as u64 != actual_size {
            return Err(PublicationResolutionError::SizeMismatch);
        }

        let mut bytes = Vec::with_capacity(actual_size as usize);
        file.take(self.max_bytes as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| PublicationResolutionError::ReadFailed)?;
        if bytes.len() > self.max_bytes {
            return Err(PublicationResolutionError::SizeLimitExceeded);
        }
        let actual_checksum = format!("{:x}", Sha256::digest(&bytes));
        if actual_checksum != expected_checksum {
            return Err(PublicationResolutionError::ChecksumMismatch);
        }
        let content =
            String::from_utf8(bytes).map_err(|_| PublicationResolutionError::InvalidUtf8)?;

        Ok(VerifiedLivingReadingPublication {
            media_type,
            content,
        })
    }
}

fn locator_path(locator: &str) -> Result<PathBuf, PublicationResolutionError> {
    let locator = locator.trim();
    if locator.is_empty() || locator.as_bytes().contains(&0) {
        return Err(PublicationResolutionError::InvalidLocator);
    }
    if let Some(path) = locator.strip_prefix("file://") {
        if !path.starts_with('/') {
            return Err(PublicationResolutionError::InvalidLocator);
        }
        return Ok(PathBuf::from(path));
    }
    if locator.contains("://") {
        return Err(PublicationResolutionError::InvalidLocator);
    }
    Ok(PathBuf::from(locator))
}

fn normalize_media_type(media_type: Option<&str>) -> Result<String, PublicationResolutionError> {
    let normalized = media_type
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .filter(|value| !value.is_empty())
        .ok_or(PublicationResolutionError::UnsupportedMediaType)?;
    match normalized.as_str() {
        "text/html" | "text/markdown" | "text/plain" => Ok(normalized),
        _ => Err(PublicationResolutionError::UnsupportedMediaType),
    }
}

fn validate_checksum(checksum: &str) -> Result<String, PublicationResolutionError> {
    let normalized = checksum.trim().to_ascii_lowercase();
    if normalized.len() != 64
        || !normalized
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PublicationResolutionError::InvalidChecksum);
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn uses_the_importer_artifact_root_contract() {
        assert_eq!(ARTIFACT_ROOT_ENV, "LIVING_READING_ARTIFACT_ROOT");
    }

    fn candidate(
        path: &Path,
        content: &[u8],
        media_type: &str,
    ) -> LivingReadingPublicationArtifactCandidate {
        LivingReadingPublicationArtifactCandidate {
            storage_provider: "filesystem".to_string(),
            object_locator: path.to_string_lossy().into_owned(),
            content_sha256: format!("{:x}", Sha256::digest(content)),
            byte_size: content.len() as i64,
            media_type: Some(media_type.to_string()),
        }
    }

    #[test]
    fn resolves_verified_html_inside_archive_root() {
        let archive = TempDir::new().expect("archive");
        let body = b"<article>Verified reading</article>";
        let path = archive.path().join("reading.html");
        fs::write(&path, body).expect("write");
        let resolver = LivingReadingPublicationResolver::new(archive.path().to_path_buf(), 1024)
            .expect("resolver");

        let verified = resolver
            .resolve(&candidate(&path, body, "Text/HTML; charset=utf-8"))
            .expect("verified publication");

        assert_eq!(verified.media_type, "text/html");
        assert_eq!(verified.content, String::from_utf8_lossy(body));
    }

    #[test]
    fn rejects_outside_root_checksum_size_type_utf8_and_provider() {
        let archive = TempDir::new().expect("archive");
        let outside = TempDir::new().expect("outside");
        let resolver = LivingReadingPublicationResolver::new(archive.path().to_path_buf(), 8)
            .expect("resolver");

        let outside_path = outside.path().join("reading.txt");
        fs::write(&outside_path, b"safe").expect("write outside");
        assert_eq!(
            resolver.resolve(&candidate(&outside_path, b"safe", "text/plain")),
            Err(PublicationResolutionError::OutsideArchiveRoot)
        );

        let path = archive.path().join("reading.txt");
        fs::write(&path, b"safe").expect("write");
        let mut wrong_checksum = candidate(&path, b"other", "text/plain");
        wrong_checksum.byte_size = 4;
        assert_eq!(
            resolver.resolve(&wrong_checksum),
            Err(PublicationResolutionError::ChecksumMismatch)
        );

        let mut wrong_size = candidate(&path, b"safe", "text/plain");
        wrong_size.byte_size = 3;
        assert_eq!(
            resolver.resolve(&wrong_size),
            Err(PublicationResolutionError::SizeMismatch)
        );

        let unsupported = candidate(&path, b"safe", "application/pdf");
        assert_eq!(
            resolver.resolve(&unsupported),
            Err(PublicationResolutionError::UnsupportedMediaType)
        );

        let invalid_utf8_path = archive.path().join("invalid.txt");
        let invalid_utf8 = [0xff, 0xfe];
        fs::write(&invalid_utf8_path, invalid_utf8).expect("write invalid utf8");
        assert_eq!(
            resolver.resolve(&candidate(&invalid_utf8_path, &invalid_utf8, "text/plain")),
            Err(PublicationResolutionError::InvalidUtf8)
        );

        let mut wrong_provider = candidate(&path, b"safe", "text/plain");
        wrong_provider.storage_provider = "s3".to_string();
        assert_eq!(
            resolver.resolve(&wrong_provider),
            Err(PublicationResolutionError::UnsupportedProvider)
        );
    }

    #[test]
    fn rejects_files_larger_than_the_strict_cap() {
        let archive = TempDir::new().expect("archive");
        let path = archive.path().join("large.md");
        let body = b"123456789";
        fs::write(&path, body).expect("write");
        let resolver = LivingReadingPublicationResolver::new(archive.path().to_path_buf(), 8)
            .expect("resolver");

        assert_eq!(
            resolver.resolve(&candidate(&path, body, "text/markdown")),
            Err(PublicationResolutionError::SizeLimitExceeded)
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape_from_archive_root() {
        use std::os::unix::fs::symlink;

        let archive = TempDir::new().expect("archive");
        let outside = TempDir::new().expect("outside");
        let target = outside.path().join("reading.html");
        fs::write(&target, b"<p>outside</p>").expect("write target");
        let link = archive.path().join("reading.html");
        symlink(&target, &link).expect("symlink");
        let resolver = LivingReadingPublicationResolver::new(archive.path().to_path_buf(), 1024)
            .expect("resolver");

        assert_eq!(
            resolver.resolve(&candidate(&link, b"<p>outside</p>", "text/html")),
            Err(PublicationResolutionError::OutsideArchiveRoot)
        );
    }

    #[test]
    fn rejects_non_file_locators_and_invalid_checksums() {
        let archive = TempDir::new().expect("archive");
        let resolver = LivingReadingPublicationResolver::new(archive.path().to_path_buf(), 1024)
            .expect("resolver");
        let mut remote = candidate(archive.path(), b"", "text/plain");
        remote.object_locator = "https://example.invalid/reading".to_string();
        assert_eq!(
            resolver.resolve(&remote),
            Err(PublicationResolutionError::InvalidLocator)
        );

        let path = archive.path().join("reading.txt");
        fs::write(&path, b"safe").expect("write");
        let mut invalid_checksum = candidate(&path, b"safe", "text/plain");
        invalid_checksum.content_sha256 = "not-a-sha256".to_string();
        assert_eq!(
            resolver.resolve(&invalid_checksum),
            Err(PublicationResolutionError::InvalidChecksum)
        );
    }
}
