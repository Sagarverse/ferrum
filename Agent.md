# MASTER BUILD PROMPT

## Build a Production-Grade Windows Data Protection Platform: FERRUM

You are a senior Windows systems architect, cybersecurity engineer, storage engineer, distributed-systems engineer, UI/UX designer, AI engineer, QA engineer, and open-source maintainer.

Build a **complete, working, Windows-native application** called:

# FERRUM

### Continuous Data Protection, Versioning, Ransomware Defense & Automated Recovery

This is NOT a simple backup application.

The goal is to create an open-source, professional-grade, Windows-first data protection platform inspired by the *concepts* of enterprise backup platforms such as Commvault, but designed for individual creators, professionals, developers, researchers, small businesses, and power users.

Do not copy proprietary UI, code, branding, or implementation from Commvault or any other product.

The final system must be:

* Local-first
* On-premises focused
* Semi-cloud / optional-cloud
* Continuous
* Versioned
* Storage efficient
* Ransomware-aware
* Recovery focused
* Low-resource
* AI-assisted
* Manually configurable
* Open-source friendly
* Windows compatible
* Modular
* Extensible
* Reliable on large files and large datasets

The application must be an actual runnable product, not a prototype mockup.

---

# 1. CORE PRODUCT IDEA

A professional has important data on their primary Windows storage.

For example:

* Video editing projects
* RAW photos
* CAD files
* Source code
* Documents
* Databases
* Research datasets
* Music projects
* VM files
* Configuration files
* Business files
* Application project folders

The user installs Ferrum.

The user selects:

* a file
* a folder
* multiple folders
* an entire drive
* project directories
* application data

Ferrum continuously watches the selected data.

Once protection is enabled:

1. Detect changes in real time.
2. Create a protected version.
3. Deduplicate changed data.
4. Synchronize safe changes to protected replicas.
5. Maintain version history.
6. Verify integrity.
7. Detect suspicious activity.
8. Protect against destructive behavior.
9. Preserve recovery points.
10. Restore data quickly when requested.

The user should be able to continue working normally without noticeable system slowdown.

---

# 2. STORAGE MODEL

The primary source is the user's existing data.

Example:

PRIMARY STORAGE
C:\Projects\VideoProject\

Ferrum maintains protected repositories on connected storage.

Example:

EXTERNAL SSD 1

* Mirror Repository
* Encrypted Repository

EXTERNAL SSD 2

* Mirror Repository
* Encrypted Repository

The architecture must support configurable storage targets rather than hardcoding two drives.

Possible layout:

SOURCE
|
+---- Replica A: normal mirror
|
+---- Replica A: encrypted protected repository
|
+---- Replica B: normal mirror
|
+---- Replica B: encrypted protected repository
|
+---- Optional emergency cloud repository

---

# 3. VERY IMPORTANT SECURITY RULE

DO NOT blindly propagate suspicious destructive changes.

Normal edits may be synchronized immediately.

Examples of normal changes:

* edit a Word document
* save a video project
* add a photo
* rename one file
* modify one configuration file
* create a new project file

These may be synchronized normally.

However, suspicious behavior such as:

* thousands of files changing rapidly
* large-scale deletion
* rapid extension changes
* high-entropy rewrites
* known ransomware patterns
* mass encryption behavior
* abnormal rename/delete bursts
* suspicious processes accessing protected repositories

must trigger:

# PROTECTION / QUARANTINE MODE

In quarantine mode:

* freeze propagation of suspicious changes
* preserve last known healthy versions
* mark current source state as suspicious
* protect healthy replicas
* create an incident record
* notify the user
* optionally isolate the suspicious process
* prepare recovery
* optionally escalate an encrypted emergency copy to cloud storage

This is one of the most important design requirements.

---

# 4. DELETE PROTECTION

The user should NOT be able to permanently remove protected information merely by deleting it normally.

Example:

User selects:

C:\Projects\ImportantProject

and enables protection.

Later the user performs:

Delete -> Empty Recycle Bin

Ferrum must preserve the protected versions.

The app should show:

"Protected data removal detected."

The user must explicitly enter Ferrum and choose:

# UNLINK / REMOVE PROTECTION

Only after explicit confirmation should permanent retention removal become possible according to the configured retention policy.

Support:

* soft delete
* protected delete
* unlink
* retention expiration
* permanent deletion
* delete confirmation
* administrator confirmation if enabled

Maintain a clear audit trail.

---

# 5. VERSIONING

Implement true version management.

Every meaningful change should create a recoverable version without storing unnecessary duplicate data.

Example:

VERSION 1
Initial project

VERSION 2
Added files

VERSION 3
Edited project

VERSION 4
Latest healthy state

The user must be able to:

* view versions
* compare timestamps
* browse version metadata
* restore a single file
* restore a folder
* restore the complete protected project
* restore initial version
* restore latest healthy version
* restore arbitrary historical version
* export a selected version
* preview version metadata

The UI must prominently provide:

### RESTORE

and at minimum offer:

* Initial Version
* Recent Version

Also provide:

* Version Timeline
* Browse All Versions

---

# 6. STORAGE EFFICIENCY

This project MUST NOT waste storage.

Implement:

* content-addressable storage
* BLAKE3 hashing
* chunking
* incremental backup
* deduplication
* compression
* metadata indexing
* changed-block detection
* intelligent retention
* garbage collection
* orphan block cleanup
* duplicate elimination

Do NOT create a complete duplicate of a multi-gigabyte file every time a small part changes.

For large files:

Example:

A 20 GB video changes by 30 MB.

Do not create another 20 GB copy.

Store only the changed chunks plus required metadata.

Use configurable chunking strategies.

Support adaptive chunk sizes where beneficial.

---

# 7. ENCRYPTION

Encrypted repositories must use modern authenticated encryption.

Recommended implementation:

* AES-256-GCM where hardware acceleration is available
* or XChaCha20-Poly1305 as a supported alternative
* BLAKE3 for integrity/content identifiers
* TLS 1.3 for network communication

Encryption keys must NEVER be stored as plain text.

Support:

* password-derived key
* secure key derivation
* OS credential protection
* Windows DPAPI
* optional recovery key
* key rotation
* secure key backup
* encrypted configuration
* encrypted metadata where appropriate

Password handling must never use plain-text storage.

---

# 8. PROTECTED REPOSITORY

Protected repositories should not behave like ordinary user folders.

Implement:

* restrictive Windows ACLs
* service-controlled access
* protected repository metadata
* integrity validation
* repository locking
* anti-tamper mechanisms
* write coordination
* repository health monitoring

Do not rely only on hiding folders.

The design must assume that malware may attempt:

* delete
* rename
* encrypt
* overwrite
* corrupt
* modify
* truncate
* mass-copy

protected data.

---

# 9. WINDOWS ARCHITECTURE

Build specifically for Windows.

Use a layered architecture.

Recommended stack:

## Frontend

Prefer:

* C#
* .NET 8+
* WinUI 3

Alternative technologies may be proposed only if there is a strong technical reason.

## Core protection engine

Prefer:

* Rust

because Rust is well suited for:

* filesystem processing
* hashing
* concurrency
* memory safety
* storage engines
* high-performance file processing

## Windows Service

Create:

Ferrum.Service

Responsible for:

* background protection
* filesystem watching
* scheduling
* backup jobs
* repository management
* health monitoring
* recovery jobs
* resource throttling

## Optional advanced protection layer

Implement a Windows filesystem minifilter architecture for strong delete/write interception where appropriate.

The project must clearly separate:

* User Mode
* Service
* Core Engine
* Optional Kernel/Minifilter component

Do NOT compromise Windows stability.

Use the safest possible implementation.

If the minifilter is unavailable during development:

implement a reliable user-mode compatibility mode and clearly identify its reduced protection level.

---

# 10. FILE MONITORING

Implement reliable Windows filesystem monitoring.

Use appropriate Windows mechanisms such as:

* USN Journal
* ReadDirectoryChangesW
* filesystem watchers
* minifilter callbacks where required

Do NOT depend only on basic polling.

Support:

* file creation
* modification
* rename
* move
* delete
* metadata changes
* directory changes

The event processing system must be:

* asynchronous
* queued
* deduplicated
* crash recoverable

Events must not be lost when large numbers of files change.

---

# 11. PROCESS MONITORING

Ferrum should monitor relevant processes for suspicious behavior.

Track:

* process identity
* process path
* process activity
* protected-folder activity
* file modification rate
* deletion rate
* rename rate
* entropy changes
* extension changes
* suspicious execution patterns

Do not claim perfect malware detection.

Use multiple signals instead.

---

# 12. RANSOMWARE / THREAT DETECTION

Implement a layered detection system.

Use:

### Rule-based detection

Detect:

* mass file modifications
* mass deletion
* rapid extension changes
* abnormal rename bursts
* high-rate file writes
* entropy increases
* suspicious process-to-file relationships

### Behavioral scoring

Generate:

Threat Score:

0-100

Example:

0-20
Normal

21-50
Watch

51-75
Suspicious

76-100
Critical

### Detection response

LOW:

Continue normally.

MEDIUM:

Increase monitoring.

HIGH:

Quarantine suspicious propagation.

CRITICAL:

* protect healthy replicas
* pause destructive replication
* alert user
* create incident
* optionally isolate suspicious process
* prepare recovery
* trigger emergency encrypted cloud escalation if enabled

Avoid destructive automated actions unless explicitly configured by the user.

---

# 13. ANTI-TAMPER / FORCED DELETE MODE

The user's special idea:

If someone attempts to forcibly destroy protected data:

Ferrum should enter:

# PROTECTION LOCKDOWN

The application UI should become locked.

Show a secure fullscreen application overlay:

"Protected Data Security Event"

Provide:

* incident number
* detected process
* affected folders
* number of files
* current protection status
* last healthy version
* recovery availability

Require Ferrum authentication to unlock protected operations.

IMPORTANT:

Do NOT disable the entire Windows keyboard and mouse at OS level.

Only lock Ferrum's protected controls and sensitive operations.

The security service should continue protecting the repositories in the background.

---

# 14. EMERGENCY CLOUD ESCALATION

Ferrum is primarily local/on-premises.

Cloud is OPTIONAL.

Cloud is used as an emergency secondary layer.

When configured and when a serious threat is detected, support uploading encrypted protected data to:

* Google Drive
* OneDrive
* Dropbox
* Amazon S3
* Backblaze B2
* S3-compatible storage
* other providers through a plugin system

The cloud copy must be encrypted BEFORE upload.

The cloud provider should not receive plaintext user data.

Cloud upload should be resumable.

Support:

* pause
* resume
* retry
* bandwidth limiting
* encryption
* deduplication
* chunked upload
* integrity verification

Do not upload everything unnecessarily.

Only upload data required by:

* policy
* incident
* retention
* missing replica recovery

---

# 15. REPLICA HEALTH SYSTEM

Continuously verify replica health.

Each stored object/chunk must have integrity metadata.

Use BLAKE3 hashes.

Implement:

* periodic integrity scans
* on-demand integrity scans
* corruption detection
* missing chunk detection
* repository consistency checks
* automatic repair

If Replica A becomes corrupted:

DO NOT trust it.

Recover healthy data from Replica B.

Rebuild Replica A from the healthy source.

Example:

Source
Healthy

Replica A
Corrupted

Replica B
Healthy

Ferrum:

1. Detects corruption.
2. Marks Replica A unhealthy.
3. Stops using it as recovery authority.
4. Reads verified data from Replica B.
5. Repairs Replica A.
6. Validates repaired chunks.
7. Returns Replica A to healthy state.

---

# 16. LIVE SYNCHRONIZATION

Implement intelligent live synchronization.

Normal operation:

Source
↓
Change Detection
↓
Versioning
↓
Deduplication
↓
Replica A
↓
Replica B

But suspicious operations must use:

Source
↓
Detection
↓
QUARANTINE
↓
Healthy replicas protected
↓
User / policy decision

This distinction MUST exist in the architecture.

---

# 17. BACKUP SCHEDULER

The user must be able to configure policies manually like enterprise software.

Provide:

* continuous mode
* real-time mode
* scheduled backup
* hourly
* daily
* weekly
* custom schedule
* event-triggered backup
* application-triggered backup where supported
* idle-time backup
* battery-aware mode
* AC-power-only mode
* bandwidth limits
* CPU limits
* storage limits

Provide:

### Backup Policy Editor

with:

* source
* destination
* frequency
* retention
* version policy
* encryption
* deduplication
* compression
* cloud escalation
* ransomware policy
* resource limits
* notification policy

---

# 18. AI CONVERSATIONAL CONFIGURATION

This is a major feature.

The home dashboard should feel like a modern AI chat interface.

Do NOT make the user learn complicated backup terminology.

The user should be able to type:

"Protect my video editing folder."

AI asks:

"Which folder?"

User:

"D:\Projects\ClientVideos"

AI:

"I found 2.4 TB of data. Would you like continuous protection with two external SSD replicas?"

User:

"Yes."

AI configures the system.

Another example:

"Protect this drive continuously, keep 20 versions, use both external SSDs, and upload an emergency encrypted copy to Google Drive if ransomware is detected."

The AI should understand and translate natural language into a structured configuration.

---

# 19. AI CONFIGURATION SAFETY

AI must NEVER silently perform destructive operations.

Require confirmation for:

* deleting repository data
* unlinking protected data
* reducing retention
* disabling ransomware protection
* deleting versions
* changing encryption
* removing recovery points
* cloud uploads containing user data
* disabling a protection policy

Safe actions such as:

* enable monitoring
* inspect status
* show storage health
* recommend settings
* create a draft policy

can be performed automatically.

For destructive actions:

AI must generate:

ACTION PREVIEW

Then:

[ Confirm ]
[ Cancel ]

---

# 20. CONFIGURATION PANEL

Provide a clear:

# SEE CONFIGURATION

button.

When clicked, show:

* protected sources
* replica destinations
* cloud destination
* current schedule
* retention
* encryption
* deduplication
* compression
* ransomware protection
* resource limits
* storage health
* version policy
* recovery policy

Every AI-created configuration must be editable manually.

---

# 21. AI DASHBOARD DESIGN

Create a modern conversational home screen.

Layout:

LEFT:
Navigation

* Home
* Protected Data
* Backup
* Versions
* Recovery
* Storage
* Security
* Incidents
* Schedules
* Cloud
* Reports
* Settings

CENTER:
AI conversational interface

RIGHT:
Live system state

Example:

PROTECTED
12 Folders

HEALTH
Excellent

REPLICAS
2 / 2 Healthy

LATEST VERSION
2 minutes ago

THREAT STATUS
No threats detected

STORAGE
1.2 TB / 2 TB

RECOVERY
Ready

The AI chat should feel conversational rather than like a traditional enterprise form.

---

# 22. COMMVAULT-INSPIRED INFORMATION ARCHITECTURE

Take inspiration from enterprise backup concepts:

* policies
* jobs
* storage
* repositories
* retention
* recovery points
* health
* alerts
* reports

But make the UI drastically easier.

The application should feel like:

Enterprise reliability
+
Modern AI assistant
+
Simple consumer experience

---

# 23. DASHBOARD

Create a professional dashboard containing:

### Protection Score

0-100

Example:

Protection Score: 94

### Live status

* Protected folders
* Current backup activity
* Recent versions
* Repository health
* Replica status
* Cloud status

### Security

* threat status
* suspicious events
* ransomware incidents
* blocked processes
* integrity alerts

### Recovery

* latest recovery point
* oldest available version
* total recoverable data
* one-click recovery

### Storage

* source data
* deduplicated data
* compressed data
* storage saved
* replica capacity

---

# 24. RECOVERY EXPERIENCE

Make recovery extremely simple.

User selects:

PROJECT

Then:

RESTORE

Show:

### Restore Point

* Initial Version
* Latest Healthy Version
* Browse Timeline

Then:

Destination:

* Original location
* New location
* External drive

Then:

[ RESTORE ]

Provide:

* overwrite protection
* conflict handling
* integrity verification
* restore progress
* restore log

---

# 25. VERSION TIMELINE

Build a visual timeline.

Example:

2026-08-20 09:30
Initial version

2026-08-20 10:15
Edited

2026-08-20 12:44
New files

2026-08-20 15:02
Latest healthy version

2026-08-20 15:04
Suspicious encryption detected

Make versions visually distinguishable:

* Healthy
* Suspicious
* Quarantined
* Restored
* Deleted source but preserved

---

# 26. STORAGE MANAGEMENT

Provide enterprise-quality storage management.

Show:

* drive name
* drive type
* filesystem
* capacity
* free space
* health
* encryption
* repository status
* last verified
* last successful synchronization
* error count
* throughput

Support:

* repository creation
* repository repair
* repository migration
* repository expansion
* destination replacement

---

# 27. RESOURCE MANAGEMENT

This is extremely important.

The application MUST NOT destroy the user's working performance.

The user may edit:

* 4K/8K video
* large images
* CAD
* datasets
* codebases
* large archives
* virtual machines

Therefore implement adaptive resource control.

Configure:

CPU limit
Memory limit
Disk I/O priority
Maximum concurrent operations
Maximum background threads
Bandwidth limit
Battery behavior
AC power behavior

Use:

* low-priority I/O
* adaptive queues
* bounded concurrency
* backpressure
* asynchronous processing
* incremental hashing
* chunk caching
* memory pooling
* streaming

Never load enormous files entirely into RAM merely to hash or copy them.

Always stream.

---

# 28. LARGE FILE HANDLING

Must support:

* GB files
* tens of GB
* hundreds of GB
* TB-scale datasets where practical

Support:

* resumable copy
* interrupted backup recovery
* chunk-level restart
* partial progress
* temporary files
* atomic finalization

If a backup is interrupted:

DO NOT restart the entire file unnecessarily.

Resume from verified chunks.

---

# 29. APPLICATION CRASH RECOVERY

Ferrum must recover safely from:

* application crash
* Windows restart
* power failure
* external drive unplug
* network failure
* cloud upload interruption
* corrupted metadata
* partially written backup

Use:

* journaled metadata
* atomic transactions
* temporary states
* checkpoints
* transaction recovery
* repository validation

Never report a backup as successful until integrity verification passes.

---

# 30. DATABASE

Use a local database for metadata.

Recommended:

SQLite

Possible separation:

Metadata DB
Index DB
Job DB
Security event DB

Do not put huge binary file contents inside SQLite.

Store blobs/chunks in repositories.

Database must support:

* transactions
* WAL mode
* migrations
* corruption checking
* recovery

---

# 31. PROJECT STRUCTURE

Create a professional monorepo such as:

ferrum/

frontend/
service/
core/
filesystem/
repository/
crypto/
dedup/
versioning/
detection/
recovery/
scheduler/
cloud/
ai/
database/
installer/
docs/
tests/
scripts/

Suggested components:

Ferrum.UI
Ferrum.Service
Ferrum.Core
Ferrum.Repository
Ferrum.Security
Ferrum.AI
Ferrum.Cloud
Ferrum.Recovery

---

# 32. AI ENGINE

Make AI provider-independent.

Support:

* local LLM
* optional cloud LLM

Prefer local AI where possible.

Potential local integration:

Ollama

The AI must interact through structured tools.

Examples:

get_protected_folders()
get_storage_status()
create_backup_policy()
update_backup_policy()
get_versions()
restore_version()
get_security_events()
get_repository_health()
get_schedule()
set_schedule()

The LLM should NEVER directly execute arbitrary shell commands.

Use an allowlisted tool layer.

---

# 33. AI RESPONSE ENGINE

Natural language:

"How much data is protected?"

AI:

"You currently protect 1.8 TB across 7 folders. Both local replicas are healthy."

Natural language:

"Why is today's backup taking longer?"

AI checks:

* current job
* file sizes
* changed blocks
* disk speed
* CPU
* queue
* destination health

Then explains the reason.

Natural language:

"Restore the version from yesterday before 5 PM."

AI resolves the appropriate recovery point and asks for confirmation before recovery if the operation could overwrite current files.

---

# 34. SECURITY MODEL

Implement:

* least privilege
* secure IPC
* authenticated local service communication
* encrypted credentials
* DPAPI
* signed binaries where possible
* secure logging
* tamper-evident audit records
* RBAC for future multi-user version
* MFA-ready architecture
* protected secrets
* certificate management
* secure update mechanism

Do not use:

* hardcoded passwords
* hardcoded encryption keys
* insecure temporary file handling
* arbitrary shell execution from AI

---

# 35. AUDIT LOG

Record:

* user
* timestamp
* action
* source
* destination
* process
* result
* version
* policy
* security event
* recovery operation

Logs must be searchable.

---

# 36. OPEN-SOURCE REQUIREMENTS

Prepare the project for GitHub.

Include:

README.md

LICENSE

CONTRIBUTING.md

SECURITY.md

ARCHITECTURE.md

ROADMAP.md

CHANGELOG.md

docs/

examples/

tests/

GitHub issue templates.

GitHub pull-request template.

Code of conduct.

Developer setup instructions.

---

# 37. TESTING REQUIREMENTS

Do NOT stop after coding.

Create:

### Unit tests

For:

* hashing
* chunking
* deduplication
* versioning
* encryption
* metadata
* repository logic

### Integration tests

For:

* source -> replica
* source -> replica -> restore
* corruption -> repair
* delete -> recovery
* ransomware simulation -> quarantine
* interrupted backup -> resume

### Stress tests

Test:

* thousands of files
* millions of filesystem events
* large files
* simultaneous changes
* rapid modifications
* drive disconnect
* power interruption simulation

### UI tests

Test:

* onboarding
* configuration
* restore
* alerts
* AI workflow

---

# 38. FAILURE SCENARIOS

The system must gracefully handle:

1. External SSD disconnected.
2. Replica becomes full.
3. Repository corruption.
4. User changes source while backup runs.
5. File locked by another application.
6. File deleted.
7. File renamed.
8. File moved.
9. Video editor writes temporary files.
10. Massive folder changes.
11. Ransomware simulation.
12. Windows reboot during backup.
13. Power failure.
14. Cloud unavailable.
15. Network lost.
16. Database interrupted.
17. Application crash.
18. Service restart.

---

# 39. VIDEO EDITOR / CREATOR OPTIMIZATION

This is a major target use case.

Support applications such as:

* Adobe Premiere Pro
* DaVinci Resolve
* After Effects
* Photoshop
* Blender
* CAD tools
* IDEs
* research software

Do not unnecessarily back up:

* cache files
* temporary files
* preview files
* render caches

Allow exclusion policies.

Example AI command:

"Protect my Premiere project but ignore cache files."

The AI should configure:

Protected:
Projects
Source Media
Project Files

Excluded:
Cache
Preview
Temporary render data

---

# 40. SMART FILE CLASSIFICATION

Optionally classify data:

* critical
* important
* standard
* temporary
* cache

Use this classification to optimize:

* retention
* backup frequency
* cloud escalation
* storage priority

Never automatically delete important data merely because it is classified.

---

# 41. SMART RETENTION

Support:

* number of versions
* days
* weeks
* months
* yearly snapshots
* event-based retention

Example:

Keep:

every version for 24 hours

daily versions for 30 days

weekly versions for 6 months

monthly versions for 2 years

Make retention customizable.

---

# 42. DEDUPLICATION ENGINE

Content-based deduplication:

Input:

File A
File B

If chunks are identical:

Store once.

Maintain metadata references.

Use chunk hashes.

Garbage collection must only delete chunks that have zero active references.

---

# 43. COPY-ON-WRITE / SAFE VERSIONING

Never overwrite a trusted version directly.

Create:

new version
verify
commit
index

Only then update the "latest healthy" pointer.

This ensures an interrupted operation cannot destroy the previous good version.

---

# 44. HEALTH ENGINE

Create a single health subsystem that reports:

* source health
* repository health
* replica health
* encryption health
* database health
* cloud health
* scheduler health
* service health
* protection health

Overall:

# METALLIC HEALTH SCORE

---

# 45. NOTIFICATIONS

Support:

* Windows notifications
* email
* in-app notifications

Examples:

"Replica 2 disconnected."

"Protection paused because external SSD is unavailable."

"Suspicious encryption behavior detected."

"Recovery point created."

"Emergency cloud backup completed."

"Repository repaired."

---

# 46. OFFLINE-FIRST DESIGN

The application MUST continue functioning even without internet access.

Core functionality must remain available:

* monitoring
* versioning
* local replication
* recovery
* ransomware detection
* repository health
* scheduling

Cloud should be optional.

---

# 47. OPTIONAL CLOUD PROVIDER ABSTRACTION

Create an interface:

ICloudProvider

Implement providers through adapters.

Example:

GoogleDriveProvider
OneDriveProvider
DropboxProvider
S3Provider
BackblazeProvider

Do not tightly couple the core system to any provider.

---

# 48. INSTALLER

Create a Windows installer.

Installer should support:

* UI installation
* service installation
* optional driver installation
* repository initialization
* secure permissions
* uninstall
* upgrade

Never delete user repositories automatically during uninstall.

---

# 49. FIRST-RUN EXPERIENCE

When opened for the first time:

Show:

"Welcome to Ferrum"

Then ask:

"What would you like to protect?"

Allow:

* folder picker
* file picker
* drive picker

Then ask:

"Where should your protected copies be stored?"

Detect connected external drives.

Recommend:

Replica A
Replica B

Then ask:

"How strongly should I protect this data?"

Modes:

### Essentials

Simple continuous protection.

### Professional

Versions + ransomware detection + multiple replicas.

### Maximum Protection

Multiple replicas + encryption + advanced detection + emergency cloud escalation.

Also provide:

CUSTOM MODE

---

# 50. AI SETUP EXAMPLE

User:

"Protect my D drive project folder. I edit videos every day. Keep all changes for one week, then daily versions for 60 days. Use both SSDs. Encrypt the second copy. If you detect ransomware, protect the last healthy version and send an encrypted emergency copy to Google Drive."

AI should parse this into a structured policy.

Show:

SOURCE
D:\Projects

REPLICATION
SSD 1
SSD 2

ENCRYPTION
Enabled

VERSIONING
Continuous

RETENTION
1 week continuous
60 days daily

RANSOMWARE DETECTION
Enabled

EMERGENCY CLOUD
Google Drive

Then:

[ APPLY CONFIGURATION ]

---

# 51. MANUAL CONFIGURATION

Everything the AI can configure must also be manually configurable.

Do not hide advanced controls.

Provide:

* Basic
* Advanced
* Expert

settings.

---

# 52. PERFORMANCE TARGETS

The application must be designed to minimize:

* CPU usage
* memory usage
* unnecessary disk reads
* unnecessary disk writes
* network usage

Use configurable target limits.

Example:

Idle CPU target:
< 3-5% under normal monitoring

Backup CPU:
user configurable

Memory:
bounded and monitored

Never assume a fixed number is valid for every machine.

Measure actual system capacity and adapt.

---

# 53. OBSERVABILITY

Add internal metrics:

* events/sec
* files processed/sec
* MB/sec
* chunks processed
* dedup ratio
* compression ratio
* queue depth
* memory usage
* CPU usage
* recovery time
* error rate

Display appropriate information in a diagnostic page.

---

# 54. SECURITY INCIDENT SCREEN

Create a professional incident UI.

Example:

# THREAT DETECTED

Severity:
CRITICAL

Process:
example.exe

Affected files:
12,482

Protected versions:
Safe

Replication:
Paused

Healthy replica:
SSD 2

Emergency cloud:
Uploading...

Actions:

[ Restore Latest Healthy Version ]
[ View Affected Files ]
[ Isolate Process ]
[ Open Incident Details ]

---

# 55. RECOVERY PRIORITY

Recovery should prefer:

1. verified healthy local replica
2. alternate healthy replica
3. encrypted repository
4. cloud recovery

Recovery source selection should be automatic but visible.

---

# 56. ZERO-TRUST APPROACH

Do not assume a source is healthy merely because it is the original source.

Every recovery source must pass integrity verification.

Trust should be based on:

* version health
* integrity
* threat state
* timestamps
* repository health
* security status

---

# 57. USER EXPERIENCE PRINCIPLE

The user should be able to accomplish most important operations in under three steps.

Example:

Protect data:

1. Select folder
2. Confirm policy
3. Protect

Restore:

1. Select data
2. Select version
3. Restore

Configure:

1. Ask AI
2. Review
3. Apply

---

# 58. UI VISUAL DESIGN

Create a premium desktop application.

Design language:

* enterprise
* modern
* clean
* technical
* trustworthy
* minimal
* dark/light mode
* excellent typography
* subtle animations
* clear information hierarchy

Do NOT create a generic CRUD application.

Use:

* cards
* health indicators
* charts
* timeline views
* status badges
* storage visualizations
* progress indicators
* contextual actions

---

# 59. HOME SCREEN

The home screen should feel like an AI assistant.

Top:

FERRUM

Center:

"How can I protect your data?"

Chat interface.

Suggested actions:

"Protect a folder"

"Check backup health"

"Show recent versions"

"Recover deleted files"

"Create a backup policy"

Right side:

Protection Score
Replica Health
Threat Status
Storage
Recent Recovery Point

---

# 60. NO FAKE FUNCTIONALITY

Never implement buttons that merely show fake success messages.

Every important UI control must map to a real backend capability.

If a feature is not implemented:

* clearly mark it
* add a TODO
* do not pretend it works

---

# 61. DOCUMENTATION

Create detailed documentation for:

* installation
* architecture
* development
* repository format
* encryption
* AI
* ransomware detection
* recovery
* cloud escalation
* troubleshooting
* security model
* limitations

---

# 62. SECURITY LIMITATION TRANSPARENCY

Never claim:

"100% ransomware proof"

or

"cannot be hacked"

Instead communicate:

"designed to significantly improve resilience against ransomware, accidental deletion, corruption, and hardware failure."

Security claims must be evidence-based.

---

# 63. GITHUB QUALITY

The final repository should look like a serious open-source project.

Include:

* architecture diagrams
* screenshots
* demo GIF/video instructions
* setup guide
* contribution guide
* issue labels
* roadmap
* security policy
* release process

---

# 64. DEVELOPMENT PHASES

Build in phases.

## PHASE 1

Core Windows application

* source selection
* filesystem monitoring
* local repository
* basic copy
* hashing

## PHASE 2

Versioning

* incremental versions
* chunking
* deduplication
* restore

## PHASE 3

Multiple replicas

* replica manager
* repository health
* integrity repair

## PHASE 4

Security

* ransomware detection
* suspicious-change quarantine
* delete protection
* incident management

## PHASE 5

Advanced storage

* encryption
* compression
* retention
* garbage collection

## PHASE 6

Cloud escalation

* encrypted cloud upload
* provider abstraction
* resume/retry

## PHASE 7

AI assistant

* chat UI
* tool calling
* natural language policies
* configuration preview

## PHASE 8

Advanced enterprise functionality

* reporting
* advanced scheduling
* policy management
* audit
* diagnostics

---

# 65. ACCEPTANCE TEST

Do not consider the project complete until this scenario works:

1. Create a test folder.
2. Put 100+ files inside.
3. Enable Ferrum.
4. Create first recovery point.
5. Modify files.
6. Add files.
7. Rename files.
8. Delete files.
9. Verify versions.
10. Disconnect Replica A.
11. Continue modifying files.
12. Reconnect Replica A.
13. Verify synchronization.
14. Corrupt selected replica data artificially.
15. Detect corruption.
16. Repair replica from healthy copy.
17. Simulate mass encryption behavior.
18. Confirm suspicious behavior is quarantined.
19. Confirm healthy replica remains usable.
20. Restore latest healthy version.
21. Restore initial version.
22. Reboot Windows during an active operation.
23. Confirm recovery after restart.
24. Confirm no data loss in the repository.
25. Confirm AI can configure a policy.
26. Confirm manual configuration remains available.

---

# 66. REQUIRED DELIVERABLES

Deliver:

1. Complete source code.
2. Windows executable.
3. Windows installer.
4. Windows service.
5. Repository engine.
6. Versioning engine.
7. Deduplication engine.
8. Encryption engine.
9. Security engine.
10. Recovery engine.
11. Scheduler.
12. Replica manager.
13. Optional cloud adapters.
14. AI assistant.
15. Professional dashboard.
16. Documentation.
17. Tests.
18. Architecture diagrams.
19. GitHub-ready repository.
20. Development instructions.

---

# 67. CODING PRINCIPLES

Use:

* clean architecture
* modular design
* interfaces
* dependency injection
* asynchronous programming
* structured logging
* typed models
* error handling
* retries
* cancellation
* resource limits
* secure defaults
* testability

Avoid:

* giant classes
* giant functions
* global mutable state
* hardcoded paths
* hardcoded passwords
* hardcoded cloud credentials
* arbitrary shell commands
* UI-bound storage logic
* blocking filesystem operations on UI thread

---

# 68. FINAL COMMAND

Do not merely generate a conceptual design.

Build the actual application.

Start by producing:

1. architecture
2. repository structure
3. data models
4. security model
5. service boundaries
6. API contracts
7. implementation plan

Then implement the project incrementally.

After each subsystem:

* compile
* test
* fix errors
* run integration tests
* verify behavior

Never hide errors.

Never skip tests.

Never claim a feature is complete unless it actually works.

Prioritize:

DATA SAFETY
SECURITY
CORRECTNESS
RECOVERY
PERFORMANCE
SIMPLICITY

above visual effects.

The final product should feel like:

# COMMVAULT-LEVEL DATA PROTECTION CONCEPTS

*

# LOCAL / ON-PREMISES FIRST

*

# SMART REPLICATION

*

# RANSOMWARE-AWARE SECURITY

*

# VERSIONED RECOVERY

*

# AI CONVERSATIONAL CONFIGURATION

*

# SIMPLE MODERN WINDOWS EXPERIENCE

The final result should be a strong open-source foundation that can eventually grow into a professional data-protection platform for creators, developers, professionals, and small organizations.

Project name:

# FERRUM

Primary tagline:

### "Protect Once. Recover Anytime."
