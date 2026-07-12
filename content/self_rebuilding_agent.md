What would it take for an agent to add a tool to itself, test the change, reload its own program, resume the original objective, and then use the new tool?

We built that loop in tetherscript and tested it on Windows with a concrete result: a running agent added a new `system_indicator` tool to its own `.tether` source, passed a compiler check, hot-reloaded in the same host process, resumed from persisted context, called the new tool, wrote `SELF IMPROVEMENT PROVEN` to a file, and opened that file in Notepad.

This is not an agent editing an external plugin and asking a human to restart it. The editable agent runtime is itself written in tetherscript.

## The experiment

The agent was given this objective in an isolated workspace:

> Add a model-visible `system_indicator` tool to your own running sidecar source. The tool must write a supplied message to `self_improvement_indicator.txt` and open it in Notepad on Windows. Test the underlying behavior, validate the edited source, reload, resume automatically, and invoke the new tool with the exact message `SELF IMPROVEMENT PROVEN`.

The model was selected with the exact specification:

```text
openai-codex/gpt-5.6-sol-fast:high
```

That exact user-facing model specification is preserved in the proof workspace's model state. Provider credentials stayed behind a capability boundary: the `.tether` program owns provider and model selection policy, while the Rust host exposes only a credential-safe `vault.provider(id)` operation to authority scripts.

## What the agent changed

The running agent inspected its own `agent_tui.tether` sidecar and made three focused changes.

First, it added a model and MCP tool definition:

```tether
p = map()
p.message = prop("Message to write to self_improvement_indicator.txt and display in Notepad on Windows.")
defs.push(tool_for(
    kind,
    "system_indicator",
    "Write a workspace self-improvement indicator and display it in Notepad on Windows.",
    p,
    ["message"]
))
```

Second, it added the implementation to `run_tool`:

```tether
if name == "system_indicator" {
    let path = "self_improvement_indicator.txt"
    let message = arg(args, "message", "")
    let written = fs_write(path, message)
    if written.is_err() {
        return "system_indicator failed: " + written.err()
    }
    if process_platform() == "windows" {
        let command = "Start-Process notepad.exe -ArgumentList (Resolve-Path -LiteralPath 'self_improvement_indicator.txt').Path"
        let launched = powershell(command, tool_timeout_ms(args))
        if launched.is_err() {
            return "system_indicator launch failed: " + launched.err()
        }
        let report = launched.unwrap()
        if !arg(report, "success", false) {
            return "system_indicator launch failed: " + check_output(report)
        }
    }
    return "system indicator wrote " + path
}
```

Finally, it added `system_indicator` to the router allow-list so a later model generation could call it.

The generated patch was preserved rather than discarded. That matters: evidence should remain inspectable after an autonomous run.

## The reload boundary

A self-modifying agent cannot safely mutate the instructions currently executing and pretend the active model turn immediately gained a new tool. The implementation uses an explicit generation boundary:

```text
generation N
  -> inspect source
  -> edit source
  -> test behavior
  -> compiler check
  -> persist session and objective
  -> write reload marker
  -> return control to host

host
  -> consume reload marker
  -> read updated sidecar
  -> compile updated source
  -> start generation N+1 in the same process

generation N+1
  -> load persisted continuation prompt
  -> receive the new tool schema
  -> invoke the new tool
  -> verify its result
```

The compiled launcher accepts `--reload-source`, watches the sidecar selected for reload, and consumes `.tetherscript/reload` only when the marker matches that source. The VM is rebuilt from the changed source without requiring the user to close and restart the host executable.

Before requesting reload, the agent runs `tetherscript check` against the candidate. A failed check restores the previous source. Packaged agents can set `TETHERSCRIPT_BIN` so the sidecar invokes a real tetherscript compiler rather than mistakenly treating the packaged agent executable as the compiler CLI.

## Continuity across generations

Reloading code is only half the problem. The new generation must know why it exists.

Before reload, `agent_tui.tether` persists:

- the conversation session;
- the last user objective;
- a continuation prompt;
- a bounded continuation budget;
- the next feature or checkpoint to consider.

After reload, generation N+1 consumes that state and receives a prompt beginning with:

```text
The TUI reloaded after source changes.

Current objective: ...

Continue from the reloaded state.
```

This makes the workflow self-reinforcing without making it unbounded. Continuation has an explicit budget and can be paused or stopped.

## What the proof recorded

The isolated run produced a chronological event record:

1. The original self-improvement objective was submitted.
2. The model used `replace` tool calls to edit its own sidecar.
3. The candidate passed `tetherscript check`.
4. The event log recorded `source changed; restarting`.
5. The host consumed the reload marker.
6. The next generation received the persisted objective.
7. Session row 43 called the newly added tool:

```json
{
  "name": "system_indicator",
  "arguments": {
    "message": "SELF IMPROVEMENT PROVEN"
  }
}
```

8. Session row 44 returned the correlated tool result:

```text
system indicator wrote self_improvement_indicator.txt
```

Both rows used the same tool call ID:

```text
call_5qfpPx0dlcvlUe4wJ97R54gq
```

The resulting file contained:

```text
SELF IMPROVEMENT PROVEN
```

Windows desktop inspection also found a real visible window titled:

```text
self_improvement_indicator.txt - Notepad
```

## What this proves—and what it does not

The experiment proves, at a local runtime level, that a tetherscript agent can:

- inspect and edit its own sidecar source;
- add a new model-visible tool;
- test the underlying operating-system behavior;
- reject syntactically invalid candidates;
- request and survive a same-process hot reload;
- resume the original objective in a new VM generation;
- call the newly added tool;
- produce and display an observable Windows result.

It does **not** prove that every self-generated change is correct. A compiler check catches invalid syntax and ownership errors, not flawed business logic. Reliable autonomous improvement still needs focused behavioral tests, bounded authority, checkpoints, auditable changes, and explicit confirmation for destructive or external side effects.

It also does not imply that arbitrary scripts receive Vault access. The `vault` capability is granted only to authority/full-access scripts, and it returns provider capabilities without exposing credential values.

## Why tetherscript is a useful substrate

Most agent systems put the orchestration loop in a large host framework. Here, the orchestration policy remains inspectable in one `.tether` program:

- tool definitions;
- tool dispatch;
- provider/model selection;
- session persistence;
- self-check and rollback;
- hot-reload decisions;
- continuation policy.

The Rust host supplies narrow runtime primitives: compilation, VM execution, process and filesystem capabilities, provider capabilities, and the embedded reload loop. The agent decides how to compose them in tetherscript.

That separation is important. The language runtime enforces the capability boundary; the agent program owns the policy. A user can read the policy that grants the model its tools, modify it, and watch the same running host load the new generation.

## Reproducing the pattern

A packaged authority agent can be launched with an editable sidecar and an explicit compiler:

```text
set TETHERSCRIPT_BIN=C:\path\to\tetherscript.exe
agent.exe --reload-source agent_tui.tether --grant-fs .
```

Inside the TUI, models can be listed and selected with:

```text
/models
/model openai-codex/gpt-5.6-sol-fast:high
```

A self-improvement objective can be bounded with the existing continuation commands, for example:

```text
/self Add one small tool, test it, reload, use it, and stop.
```

For production use, run the experiment in an isolated workspace, keep destructive confirmation enabled, test external integrations against local sinks, and preserve the generated patch and event logs.

## Evidence

The proof run preserved these local artifacts:

```text
target/self-improve-proof/evidence.json
target/self-improve-proof/notepad-proof.png
target/self-improve-proof/agent-generated.patch
target/self-improve-proof/proof.stdout
target/self-improve-proof/proof.stderr
target/self-improve-proof/.tetherscript/agent_tui_session.jsonl
target/self-improve-proof/.tetherscript/agent_tui_events.jsonl
target/self-improve-proof/self_improvement_indicator.txt
```

The `target/` directory is intentionally not committed. The durable source, focused regression tests, and this account of the experiment are committed; the raw local artifacts remain available in the validating workspace.

## The result

The result is not an agent that magically rewrites itself without constraints. It is more useful than that: an agent with an explicit, inspectable, testable improvement cycle.

It can propose a change, implement it in its own tetherscript source, validate the candidate, cross a controlled reload boundary, recover its objective, and prove that the new generation can do something the old generation could not.

On this run, that something was simple and visible:

```text
SELF IMPROVEMENT PROVEN
```
