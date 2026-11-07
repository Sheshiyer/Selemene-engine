/** Shared account/subject contract v1. All private caches key by account + subject + revision. */
export interface AccountContext {
  schema: "noesis.account-context.v1";
  account: { id: string; email: string | null; full_name: string | null; preferred_email: string | null };
  default_subject_id: string | null;
  subjects: Array<{ id: string; key: string; display_name: string; can_read: boolean; can_edit: boolean; can_calculate: boolean; can_manage_enrollment?: boolean; profile_revision: number }>;
  relationships: Array<{ id: string; key: string; label: string | null; member_subject_ids: string[]; can_read: boolean }>;
}
export interface SubjectProfileFields {
  full_name: string | null;
  birth_date: string | null;
  birth_time: string | null;
  birth_location: {lat: number; lng: number; name: string} | null;
  timezone: string;
  preferences: Record<string, unknown>;
  birth_time_confidence: "unspecified" | "unknown" | "unverified" | "approximate" | "exact" | "recorded";
  source_hashes: Record<string,string>;
}
export interface SubjectProfile { schema: "noesis.subject-profile.v1"; subject_id: string; revision: number; profile: SubjectProfileFields }
export interface RegisteredSession { access_token: string; refresh_token: string; token_type: "Bearer"; expires_in: number }
export interface EmailLinkChallenge { status: "pending_verification"; challenge: string; expires_in: number; complete_path: string; verification_path: string }
export interface SubjectCalculation { expected_revision: number; options?: Record<string,unknown>; current_time?: string }
export interface SubjectReading { id: string; subject_id: string; actor_account_id: string; profile_revision: number; result: unknown }
/** Invitation grants only explicitly selected, owned relationships containing this subject. */
export interface EnrollmentCreateRequest {
  subject_id: string;
  intended_email?: string;
  sharing_reader_id?: string;
  expires_in_seconds?: number;
  relationship_ids?: string[];
}
export interface EnrollmentPreview {
  id: string;
  subject_id: string;
  display_name: string;
  sharing_reader_id: string | null;
  sharing_reader_name: string | null;
  relationships: Array<{id: string; label: string | null}>;
  requires_owner_confirmation: boolean;
  expires_at: string;
}
