session-name := "battle-for-net"

_:
  @just -l

_new-window NAME CMD:
  tmux new-w -t "={{session-name}}" -n "{{NAME}}"
  [[ "{{CMD}}" != "" ]] && tmux send-keys -t "{{session-name}}:{{NAME}}" "{{CMD}}" ENTER || true

_new-tmux:
  tmux new -ds "{{session-name}}" -n "README"
  tmux send-keys -t "={{session-name}}:README" 'nv ./README.md "+set wrap"' ENTER
  @just _new-window "Edit" ""
  @just _new-window "Check" "cargo check"
  @just _new-window "Serve" "just serve"
  @just _new-window "Git" "git status"
  @just _new-window "Misc" ""

tmux:
  tmux has-session -t "={{session-name}}" || just _new-tmux
  tmux a -t "={{session-name}}"

gh_watch:
  zsh -c 'gh run watch -i 1 --exit-status && echo "RUN SUCCESFUL" || gh run view --log-failed'

serve:
  WEBKIT_DISABLE_COMPOSITING_MODE=1 /usr/bin/dx serve --platform desktop --hot-reload true --verbose # --hot-patch

run:
  WEBKIT_DISABLE_COMPOSITING_MODE=1 /usr/bin/dx run --platform desktop --verbose # --hot-patch

release-run:
  WEBKIT_DISABLE_COMPOSITING_MODE=1 /usr/bin/dx run --platform desktop --verbose --release

check:
  /usr/bin/dx check --platform desktop
