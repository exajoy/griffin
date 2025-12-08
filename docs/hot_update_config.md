## Explain

Griffin proxy can detect changes in its configuration file
while it is running, and apply those changes without
restarting the process.

+---------------+ +----------------+ +--------------------+
| YAML file | --FS--> | Config Watcher | --mpsc->| Reload Task |
+---------------+ +----------------+ +--------------------+
|
v
Parse YAML → Config
|
v
Atomic swap via ConfigStore
|
v
ListenerManager reloads TCP listener
