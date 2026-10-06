# Notifications and event routing

Open **View → Sound & Alerts → Inbox**, or search Notifications in Cmd/Ctrl+K.
The inbox reads the existing simulation Chronicle. It adds no synthetic alerts or hazard models.
Search titles/details, filter severity/category, show unread entries, mark entries read, jump to a
surviving object or open the Chronicle.

## Delivery preferences

Routing uses the simulation's eight existing event categories. Every category has independent
**Inbox**, **Popup**, **Sound**, **Pause** and **minimum priority** controls. Priority is the existing
1–5 importance value; 5 means a milestone. The threshold gates interruptions; the inbox can retain
lower-priority events. Disable all routes for Chronicle-only delivery. No simulation event is deleted.
Pause is opt-in for each category and defaults off.

Do Not Disturb suppresses popups, alert sounds and configured pause actions. Photo capture also
suppresses interruptions. Events routed to the inbox remain accessible. Master/alert volume and
Mute all alert sounds can silence sounds independently of delivery. Critical-only sound uses the
severity grouping below. Milestone auto-slowdown remains a separate existing setting.

Popups are limited to four; a burst produces at most one alert bell per four seconds. Existing
popups are dismissed by quiet/photo mode. Routing preference changes affect future events.

## Meaning and storage

Severity labels group existing events: Disaster/War priority 5 is Critical; their priority 3–4 is
Warning; other priority 4–5 events are Notice; remaining events are Information. These are UI labels,
not newly validated physical emergency thresholds.

The inbox retains the most recent 500 routed events. The saved universe's Chronicle remains the
source of durable history. Opening a universe imports its recent history silently and read; load,
undo and rewind reset inbox read marks. New events are detected using the last event rather than
only a count, so pruning old minor events does not hide new arrivals. If the last event has itself
been pruned, remaining events are imported; the source currently has no stable event IDs.

Preferences live in the app settings, outside Universe. Save preferences persists the routing and
reports write errors. The inbox itself is an app view; it is not a second authoritative history.
Future spacecraft, settlement, dataset and research notification categories require actual producers
before being added. Per-category custom sounds and persisted read marks remain future work.
