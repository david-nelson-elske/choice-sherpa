# Decision Profile Implementation Checklist

**Feature:** Cross-Decision Intelligence (Decision Profile)
**Module:** user (new)
**Priority:** P3 (Phase 4 of Agent-Native Enrichments)
**Specification:** [features/user/decision-profile.md](../features/user/decision-profile.md)
**Created:** 2026-01-09

---

## Overview

This checklist tracks implementation of the Decision Profile feature - a persistent, user-owned artifact that captures decision-making patterns, risk tolerance, preferences, and tendencies across multiple sessions to enable personalized AI guidance.

### Key Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    DECISION PROFILE                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│   │ Risk Profile │  │   Values &   │  │  Decision    │         │
│   │              │  │  Priorities  │  │    Style     │         │
│   └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│   │ Blind Spots  │  │Communication │  │  Decision    │         │
│   │  & Growth    │  │    Prefs     │  │   History    │         │
│   └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Foundation

### Domain Layer - Core Types

- [ ] `domain/user/mod.rs` - User module setup
  - [ ] Module exports
  - [ ] Re-exports for public API

- [ ] `domain/user/profile.rs` - DecisionProfile aggregate
  - [ ] DecisionProfileId value object
  - [ ] DecisionProfile aggregate root
  - [ ] ProfileVersion value object
  - [ ] ProfileConfidence enum (Low, Medium, High, VeryHigh)
  - [ ] ProfileConsent struct with timestamps
  - [ ] Domain invariants implementation
  - [ ] Unit tests

- [ ] `domain/user/risk_profile.rs` - Risk assessment types
  - [ ] RiskClassification enum (RiskSeeking, RiskNeutral, RiskAverse)
  - [ ] RiskProfile struct with confidence
  - [ ] RiskDimensions struct (financial, career, temporal, relational, health)
  - [ ] RiskScore value object (1-5 scale with confidence)
  - [ ] RiskEvidence struct
  - [ ] RiskIndicatorType enum
  - [ ] Unit tests

- [ ] `domain/user/values.rs` - Values and priorities types
  - [ ] ValuesPriorities struct
  - [ ] ConsistentObjective struct (name, frequency, weight)
  - [ ] ObjectiveWeight enum (Low, Medium, High, Critical)
  - [ ] ValueTension struct
  - [ ] DecisionDomain enum (Career, Financial, Family, etc.)
  - [ ] Unit tests

- [ ] `domain/user/decision_style.rs` - Decision-making style types
  - [ ] DecisionMakingStyle struct
  - [ ] StyleClassification enum (AnalyticalCautious, etc.)
  - [ ] StyleDimensions struct
  - [ ] DimensionScore struct
  - [ ] CognitivePattern struct
  - [ ] CognitiveBiasType enum (Anchoring, LossAversion, etc.)
  - [ ] Unit tests

- [ ] `domain/user/blind_spots.rs` - Blind spots and growth
  - [ ] BlindSpotsGrowth struct
  - [ ] BlindSpot struct
  - [ ] GrowthObservation struct
  - [ ] Unit tests

- [ ] `domain/user/communication.rs` - Communication preferences
  - [ ] CommunicationPreferences struct
  - [ ] InteractionStyle struct
  - [ ] PreferenceLevel enum
  - [ ] ChallengeStyle enum
  - [ ] PacingPreference enum
  - [ ] UncertaintyStyle enum
  - [ ] Unit tests

- [ ] `domain/user/history.rs` - Decision history types
  - [ ] DecisionHistory struct
  - [ ] DecisionRecord struct
  - [ ] OutcomeRecord struct
  - [ ] SatisfactionLevel enum
  - [ ] DomainStats struct
  - [ ] PredictionAccuracy struct
  - [ ] Unit tests

### Domain Events

- [ ] `domain/user/events.rs` - Profile domain events
  - [ ] DecisionProfileCreated
  - [ ] DecisionProfileUpdated
  - [ ] RiskProfileRecalculated
  - [ ] BlindSpotIdentified
  - [ ] GrowthObserved
  - [ ] OutcomeRecorded
  - [ ] ProfileDeleted
  - [ ] ProfileExported

### Database

- [ ] `migrations/2026MMDD_create_decision_profiles.sql`
  - [ ] decision_profiles table
    - [ ] id, user_id (unique)
    - [ ] file_path, content_checksum
    - [ ] version, profile_confidence
    - [ ] risk_profile JSONB
    - [ ] values_priorities JSONB
    - [ ] decision_style JSONB
    - [ ] blind_spots_growth JSONB
    - [ ] communication_prefs JSONB
    - [ ] decisions_analyzed count
    - [ ] consent JSONB
    - [ ] Timestamps
  - [ ] profile_decision_history table
    - [ ] id, profile_id, cycle_id
    - [ ] decision_date, title, domain
    - [ ] dq_score, key_tradeoff, chosen_alternative
    - [ ] outcome fields (satisfaction, actual_consequences)
    - [ ] risk_indicators JSONB, objectives_used JSONB
  - [ ] Indexes (user, profile, domain, date)

---

## Phase 2: Storage Layer

### Ports

- [ ] `ports/profile_repository.rs` - ProfileRepository trait
  - [ ] create(profile) - Create with consent
  - [ ] update(profile) - Update existing
  - [ ] find_by_user(user_id) → DecisionProfile
  - [ ] delete(profile_id) - Full deletion
  - [ ] export(profile_id, format) → Vec<u8>

- [ ] `ports/profile_file_storage.rs` - ProfileFileStorage trait
  - [ ] write(user_id, content) → FilePath
  - [ ] read(user_id) → String
  - [ ] exists(user_id) → bool
  - [ ] delete(user_id)
  - [ ] StorageError enum

- [ ] `ports/profile_reader.rs` - ProfileReader trait
  - [ ] get_summary(user_id) → ProfileSummary
  - [ ] get_agent_instructions(user_id, domain) → AgentInstructions
  - [ ] get_decision_history(user_id, limit) → Vec<DecisionRecord>
  - [ ] ProfileSummary struct
  - [ ] AgentInstructions struct

### Adapters

- [ ] `adapters/filesystem/profile_storage.rs` - LocalProfileFileStorage
  - [ ] Base directory configuration (/profiles/{user_id}/)
  - [ ] Profile markdown file (profile.md)
  - [ ] Atomic write (temp file + rename)
  - [ ] Checksum computation
  - [ ] Unit tests

- [ ] `adapters/postgres/profile_repository.rs` - PostgresProfileRepository
  - [ ] Create with file coordination
  - [ ] Update with file sync
  - [ ] Delete with file cleanup
  - [ ] JSONB queries for components
  - [ ] Integration tests

- [ ] `adapters/postgres/profile_reader.rs` - PostgresProfileReader
  - [ ] Summary aggregation queries
  - [ ] Agent instructions generation
  - [ ] History queries with pagination
  - [ ] Integration tests

---

## Phase 3: Profile Generation

### Ports

- [ ] `ports/profile_generator.rs` - ProfileGenerator trait
  - [ ] generate_markdown(profile) → String
  - [ ] generate_section(component) → String
  - [ ] GenerationOptions struct

### Adapters

- [ ] `adapters/profile/mod.rs` - Module setup
- [ ] `adapters/profile/markdown_generator.rs` - ProfileMarkdownGenerator
  - [ ] Template engine setup (Tera/Handlebars)
  - [ ] Risk profile section template
  - [ ] Values section template
  - [ ] Decision style section template
  - [ ] Blind spots section template
  - [ ] Communication section template
  - [ ] History section template
  - [ ] Agent instructions section template
  - [ ] Unit tests

### Templates

- [ ] `adapters/profile/templates/profile.md.tera` - Base template
- [ ] `adapters/profile/templates/sections/`
  - [ ] risk_profile.md.tera
  - [ ] values_priorities.md.tera
  - [ ] decision_style.md.tera
  - [ ] blind_spots.md.tera
  - [ ] communication.md.tera
  - [ ] history.md.tera
  - [ ] agent_instructions.md.tera

---

## Phase 4: Analysis Engine

### Ports

- [ ] `ports/profile_analyzer.rs` - ProfileAnalyzer trait
  - [ ] analyze_decision(profile, cycle, conversations) → AnalysisResult
  - [ ] recalculate_risk_profile(history) → RiskProfile
  - [ ] detect_cognitive_patterns(history, conversations) → Vec<CognitivePattern>
  - [ ] identify_blind_spots(profile) → Vec<BlindSpot>
  - [ ] generate_agent_instructions(profile) → AgentInstructions
  - [ ] AnalysisResult struct

### Adapters

- [ ] `adapters/profile/analyzer.rs` - DefaultProfileAnalyzer
  - [ ] Risk classification algorithm
  - [ ] Domain-specific risk scoring
  - [ ] Value extraction from objectives
  - [ ] Pattern detection logic
  - [ ] Blind spot identification rules
  - [ ] Growth detection
  - [ ] Unit tests with sample data

### Risk Classification Algorithm

- [ ] `adapters/profile/risk_classifier.rs`
  - [ ] Choice analysis (40% weight)
    - [ ] Track high vs low variance option selection
    - [ ] Calculate variance preference ratio
  - [ ] Language pattern analysis (25% weight)
    - [ ] Risk-averse indicators ("safe", "downside", "conservative")
    - [ ] Risk-seeking indicators ("upside", "opportunity", "bold")
    - [ ] Pattern matching and scoring
  - [ ] Consequence rating analysis (20% weight)
    - [ ] Pessimism/optimism in ratings
    - [ ] Uncertainty handling patterns
  - [ ] Information seeking analysis (15% weight)
    - [ ] Questions asked before deciding
    - [ ] Time spent in analysis phases
  - [ ] Classification thresholds
    - [ ] Risk-Seeking: score > 0.6
    - [ ] Risk-Neutral: score 0.4-0.6
    - [ ] Risk-Averse: score < 0.4
  - [ ] Confidence calculation
  - [ ] Unit tests with edge cases

### Cognitive Bias Detection

- [ ] `adapters/profile/bias_detector.rs`
  - [ ] Anchoring detection
    - [ ] Track first numbers mentioned
    - [ ] Compare to final ratings
  - [ ] Loss aversion detection
    - [ ] Ratio of loss vs gain sensitivity
  - [ ] Status quo bias detection
    - [ ] Frequency of choosing "do nothing"
  - [ ] Confirmation bias detection
    - [ ] Counter-evidence seeking patterns
  - [ ] Sunk cost detection
    - [ ] Prior investment influence
  - [ ] Unit tests

---

## Phase 5: Application Layer

### Commands

- [ ] `application/commands/create_profile.rs`
  - [ ] CreateProfileCommand (user_id, consent)
  - [ ] CreateProfileHandler
  - [ ] Consent validation
  - [ ] Empty profile creation
  - [ ] File + DB coordination
  - [ ] Unit tests

- [ ] `application/commands/update_profile_from_decision.rs`
  - [ ] UpdateProfileFromDecisionCommand (user_id, cycle_id)
  - [ ] UpdateProfileFromDecisionHandler
  - [ ] Consent check
  - [ ] Cycle + conversation loading
  - [ ] Analysis execution
  - [ ] Profile update
  - [ ] File regeneration
  - [ ] Unit tests

- [ ] `application/commands/record_outcome.rs`
  - [ ] RecordOutcomeCommand (user_id, cycle_id, satisfaction, etc.)
  - [ ] RecordOutcomeHandler
  - [ ] History update
  - [ ] Prediction accuracy calculation
  - [ ] Unit tests

- [ ] `application/commands/update_consent.rs`
  - [ ] UpdateConsentCommand (user_id, consent)
  - [ ] UpdateConsentHandler
  - [ ] Consent transition logic
  - [ ] Unit tests

- [ ] `application/commands/delete_profile.rs`
  - [ ] DeleteProfileCommand (user_id, confirmation)
  - [ ] DeleteProfileHandler
  - [ ] Confirmation validation
  - [ ] File deletion
  - [ ] DB deletion
  - [ ] Audit logging
  - [ ] Unit tests

- [ ] `application/commands/export_profile.rs`
  - [ ] ExportProfileCommand (user_id, format)
  - [ ] ExportProfileHandler
  - [ ] Markdown export
  - [ ] JSON export
  - [ ] PDF export (optional)
  - [ ] Unit tests

### Queries

- [ ] `application/queries/get_profile.rs`
  - [ ] GetProfileQuery (user_id)
  - [ ] GetProfileHandler
  - [ ] Full profile loading
  - [ ] Unit tests

- [ ] `application/queries/get_profile_summary.rs`
  - [ ] GetProfileSummaryQuery (user_id)
  - [ ] GetProfileSummaryHandler
  - [ ] Lightweight summary for UI
  - [ ] Unit tests

- [ ] `application/queries/get_agent_instructions.rs`
  - [ ] GetAgentInstructionsQuery (user_id, domain)
  - [ ] GetAgentInstructionsHandler
  - [ ] Domain-specific instructions
  - [ ] Unit tests

- [ ] `application/queries/get_decision_history.rs`
  - [ ] GetDecisionHistoryQuery (user_id, limit, domain_filter)
  - [ ] GetDecisionHistoryHandler
  - [ ] Pagination support
  - [ ] Domain filtering
  - [ ] Unit tests

### Event Handlers

- [ ] `application/event_handlers/profile_update_handler.rs`
  - [ ] Handle CycleCompleted event
  - [ ] Trigger profile analysis if consent given
  - [ ] Unit tests

---

## Phase 6: HTTP Layer

### Handlers

- [ ] `adapters/http/user/profile_handlers.rs`
  - [ ] create_profile handler (POST /api/profile)
  - [ ] get_profile handler (GET /api/profile)
  - [ ] get_profile_summary handler (GET /api/profile/summary)
  - [ ] update_consent handler (PUT /api/profile/consent)
  - [ ] record_outcome handler (POST /api/profile/outcome)
  - [ ] export_profile handler (GET /api/profile/export)
  - [ ] delete_profile handler (DELETE /api/profile)
  - [ ] DTOs for all operations
  - [ ] Error mapping

### Routes

- [ ] `adapters/http/user/routes.rs` - Profile routes
  - [ ] POST /api/profile
  - [ ] GET /api/profile
  - [ ] GET /api/profile/summary
  - [ ] PUT /api/profile/consent
  - [ ] POST /api/profile/outcome
  - [ ] GET /api/profile/export?format=
  - [ ] DELETE /api/profile

### Tests

- [ ] HTTP handler tests
- [ ] Integration tests for all endpoints
- [ ] Authorization tests (user can only access own profile)

---

## Phase 7: Agent Integration

### System Prompt Enhancement

- [ ] `adapters/ai/profile_context_builder.rs`
  - [ ] Build profile context for system prompt
  - [ ] Risk guidance section
  - [ ] Blind spot prompts
  - [ ] Communication adjustments
  - [ ] Domain-specific notes
  - [ ] Suggested approach section
  - [ ] Unit tests

- [ ] `application/services/session_context_service.rs`
  - [ ] Inject profile context at session start
  - [ ] Check consent before injection
  - [ ] Handle missing profile gracefully
  - [ ] Integration tests

### Tool-Level Integration

- [ ] `adapters/ai/profile_tool_guidance.rs`
  - [ ] suggest_consequence_challenge(profile, rating)
  - [ ] suggest_objective_prompt(profile, domain)
  - [ ] suggest_alternative_exploration(profile)
  - [ ] detect_blind_spot_trigger(profile, conversation)
  - [ ] Unit tests

### Feedback Loop

- [ ] `adapters/ai/profile_feedback_extractor.rs`
  - [ ] Extract risk indicators from conversation
  - [ ] Extract communication preferences
  - [ ] Flag potential bias patterns
  - [ ] Unit tests

---

## Phase 8: Frontend - Profile Dashboard

### API Client

- [ ] `frontend/src/lib/profile/types.ts` - TypeScript types
  - [ ] DecisionProfile interface
  - [ ] RiskProfile interface
  - [ ] RiskClassification enum
  - [ ] ProfileSummary interface
  - [ ] ConsentSettings interface
  - [ ] DecisionRecord interface

- [ ] `frontend/src/lib/profile/api.ts` - API client
  - [ ] createProfile(consent)
  - [ ] getProfile()
  - [ ] getProfileSummary()
  - [ ] updateConsent(consent)
  - [ ] recordOutcome(cycleId, outcome)
  - [ ] exportProfile(format)
  - [ ] deleteProfile(confirmation)

- [ ] `frontend/src/lib/profile/stores.ts` - Svelte stores
  - [ ] profile store
  - [ ] profileSummary store
  - [ ] hasProfile derived

### Components

- [ ] `frontend/src/lib/profile/ProfileDashboard.svelte`
  - [ ] Risk profile section with radar chart
  - [ ] Values priorities chart
  - [ ] Decision style summary
  - [ ] Blind spots cards
  - [ ] Growth achievements
  - [ ] Quick stats

- [ ] `frontend/src/lib/profile/RiskProfileChart.svelte`
  - [ ] Radar/spider chart for dimensions
  - [ ] Classification badge
  - [ ] Confidence indicator
  - [ ] Dimension tooltips

- [ ] `frontend/src/lib/profile/ValuesChart.svelte`
  - [ ] Bar chart of consistent objectives
  - [ ] Frequency indicators
  - [ ] Weight badges

- [ ] `frontend/src/lib/profile/DecisionHistory.svelte`
  - [ ] Timeline view
  - [ ] Domain filters
  - [ ] DQ score indicators
  - [ ] Outcome status
  - [ ] Expandable details

- [ ] `frontend/src/lib/profile/BlindSpotCard.svelte`
  - [ ] Blind spot name and description
  - [ ] Evidence summary
  - [ ] Status indicator (active/resolved)

- [ ] `frontend/src/lib/profile/GrowthAchievement.svelte`
  - [ ] Before/after comparison
  - [ ] Trigger description
  - [ ] Achievement badge

### Routes

- [ ] `frontend/src/routes/profile/+page.svelte` - Profile dashboard
- [ ] `frontend/src/routes/profile/+page.ts` - Load function

---

## Phase 9: Frontend - Settings & Consent

### Components

- [ ] `frontend/src/lib/profile/ProfileSettings.svelte`
  - [ ] Consent toggles (collection, analysis, agent access)
  - [ ] Data visibility controls
  - [ ] Export buttons
  - [ ] Delete section with confirmation

- [ ] `frontend/src/lib/profile/ConsentManager.svelte`
  - [ ] Current consent status display
  - [ ] Toggle switches with explanations
  - [ ] Save changes with confirmation
  - [ ] Privacy policy link

- [ ] `frontend/src/lib/profile/ExportDialog.svelte`
  - [ ] Format selection (Markdown, JSON, PDF)
  - [ ] Download trigger
  - [ ] Success/error feedback

- [ ] `frontend/src/lib/profile/DeleteConfirmation.svelte`
  - [ ] Warning message
  - [ ] Type confirmation input
  - [ ] Permanent deletion warning
  - [ ] Final confirm button

### Routes

- [ ] `frontend/src/routes/profile/settings/+page.svelte`
- [ ] `frontend/src/routes/profile/settings/+page.ts`

---

## Phase 10: Frontend - Onboarding Consent

### Components

- [ ] `frontend/src/lib/profile/OnboardingConsent.svelte`
  - [ ] Benefits explanation
  - [ ] What we collect section
  - [ ] Privacy commitments
  - [ ] Granular consent options
  - [ ] Skip option
  - [ ] Continue button

- [ ] `frontend/src/lib/profile/ConsentOption.svelte`
  - [ ] Toggle with label
  - [ ] Description text
  - [ ] "Learn more" expandable

### Integration

- [ ] Post-decision consent prompt trigger
  - [ ] Check if profile exists
  - [ ] Show onboarding if first completion
  - [ ] Respect "don't ask again" preference

---

## Phase 11: Outcome Recording

### Components

- [ ] `frontend/src/lib/profile/OutcomeRecorder.svelte`
  - [ ] Satisfaction rating (1-5 stars)
  - [ ] Actual consequences text
  - [ ] Surprises list
  - [ ] "Would decide same" toggle
  - [ ] Submit button

- [ ] `frontend/src/lib/profile/OutcomePrompt.svelte`
  - [ ] Reminder for past decisions
  - [ ] Quick satisfaction rating
  - [ ] "Remind me later" option
  - [ ] "Decision resolved" dismiss

### Notifications

- [ ] Outcome reminder system
  - [ ] 30-day post-decision prompt
  - [ ] 90-day follow-up prompt
  - [ ] Snooze functionality

---

## Phase 12: Testing & Polish

### Unit Tests

- [ ] Domain entity tests
  - [ ] RiskProfile classification logic
  - [ ] ProfileConfidence calculation
  - [ ] Consent state transitions
- [ ] Risk classifier tests
  - [ ] Edge cases (no data, mixed signals)
  - [ ] Confidence calculation
- [ ] Bias detector tests
  - [ ] Each bias type
  - [ ] False positive prevention
- [ ] Command handler tests
- [ ] Query handler tests
- [ ] Profile generator tests

### Integration Tests

- [ ] Full profile lifecycle
  - [ ] Create → Update → Export → Delete
- [ ] Risk profile evolution over decisions
- [ ] Agent integration with profile context
- [ ] Consent flow scenarios
- [ ] Cross-session persistence

### E2E Tests

- [ ] Onboarding consent flow
- [ ] Profile dashboard displays correctly
- [ ] Settings changes persist
- [ ] Export downloads work
- [ ] Delete removes all data
- [ ] Agent uses profile in conversations

### Documentation

- [ ] API documentation for profile endpoints
- [ ] User guide for profile features
- [ ] Privacy documentation
- [ ] Risk classification methodology doc
- [ ] Architecture decision record (ADR)

---

## Acceptance Criteria

### Core Functionality
- [ ] User can create a profile with explicit consent
- [ ] Profile updates automatically after each decision
- [ ] Risk profile classification is accurate (validated against known patterns)
- [ ] Blind spots are surfaced based on behavioral patterns
- [ ] Communication preferences are learned and applied

### Privacy & Security
- [ ] Profile cannot be created without consent
- [ ] User can view all collected data
- [ ] User can export their profile in multiple formats
- [ ] User can delete all profile data permanently
- [ ] Profile data is not shared or used without consent

### Agent Integration
- [ ] Agent receives profile context in system prompt
- [ ] Agent adjusts behavior based on risk profile
- [ ] Agent prompts for identified blind spots
- [ ] Communication style adapts to preferences

### User Experience
- [ ] Profile dashboard provides clear insights
- [ ] Risk profile is easy to understand
- [ ] Decision history shows patterns over time
- [ ] Outcome recording is simple and non-intrusive

### Performance
- [ ] Profile loading < 200ms
- [ ] Analysis execution < 2s per decision
- [ ] Dashboard rendering < 500ms
- [ ] Risk classification < 500ms

---

## Dependencies

### External Crates

- [ ] `sha2` - Content checksums
- [ ] `tera` or `handlebars` - Template engine
- [ ] `regex` - Pattern matching for language analysis
- [ ] `statistical` or similar - Statistical calculations

### Frontend Libraries

- [ ] Chart library (Chart.js / D3.js / Svelte-specific)
- [ ] Date formatting library
- [ ] Export utilities (file-saver)

### Internal Dependencies

- [ ] Foundation module (UserId, Timestamp)
- [ ] Cycle module (CycleId, Cycle)
- [ ] Conversation module (ConversationReader)
- [ ] Decision Document (for content reference)

---

## Notes

- Start with Phase 1-5 (core functionality) before agent integration
- Risk classification requires minimum 3 decisions for meaningful results
- Privacy is paramount - default to minimal collection
- Cognitive bias detection should flag, not accuse
- Profile should be a helpful tool, not a judgment
- Consider A/B testing risk classification thresholds
- Plan for GDPR compliance from the start

---

*Checklist Version: 1.0.0*
*Last Updated: 2026-01-09*
