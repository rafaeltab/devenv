# Rafaeltab command line tool

```
rafaeltab workspace list
rafaeltab workspace current
rafaeltab workspace find
rafaeltab workspace find-tag
rafaeltab workspace tmux
rafaeltab tmux
```

```
rafaeltab workspace list
rafaeltab workspace current
rafaeltab workspace find
rafaeltab workspace find-tag
rafaeltab workspace create
rafaeltab tmux list
rafaeltab tmux start <workspace|name|EMPTY>
```

## Workspace

## Tmux


### list

Lists all started sessions, and possible to start tmux sessions.
It includes sessions that are present in configuration, but are not started.

### start

Starts the specified session, if it is not already started, either a workspace, session with name, or all (empty).
