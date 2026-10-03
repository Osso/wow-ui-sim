# pi-delegate finished-child resume proposal

Date: 2026-10-03. Proposal only. No plugin files changed; no agents, model requests, plugin tests, or validation commands run. The only CLI invocation was `pi --help`.

## 1. Root cause and host contract

Plugin root: `/home/osso-test/AgentConfig/claude-plugins/pi-delegate`.

- `hooks/register.js:218–223`: `registerNativeTaskCleanup` calls `deleteTaskBindings` on **every child `turn.complete`**. That deletes both the name reservation and the child-ID record, including the only stored original prompt.
- `hooks/register.js:245–251`: `bindNativeTask` looks up a task by the native child's name and throws `original task unavailable for selected native Sol child` when the deleted record is absent. `registerNativeSolDelegation`, at line 281, prefixes that exception with `GPT delegation failed:` and returns a refusal. This happens before `delegateConversation` and `process.spawn`; it explains the immediate failure without a worker.
- `hooks/register.js:193–207`: fresh `agent.spawn` saves `event.prompt`, then binds the native child ID. Continuing an existing finished child is not a fresh spawn that reconstructs this reservation. The cleanup policy, not a missing model capability, destroys the needed state.
- `hooks/register.js:81–126`: each Pi worker is ephemeral (`--no-session --no-supervisor`), and its session identity is never retained. Its stdin includes the original task and selected native API conversation. There is no saved Pi session to resume.
- Existing tests deliberately enforce the defect: the lifecycle test beginning at `tests/delegate.test.ts:329` expects refusal after `turn.complete`; the early-completion test beginning at line 231 also expects a later step to refuse. These expectations must change, not merely gain a separate happy-path test.

### Locally inspected API documentation

Searched the plugin, `~/.claude/plugins`, `~/.claude/skills/.system`, and `~/.claude/dev-mods`; no extracted plugin-authoring skill appeared there. Inspected the installed Claude executable at `/home/osso-test/.local/share/claude-code/2.1.288/package/claude`, without executing it. Its embedded, Zstandard-compressed `plugin-authoring` SKILL.md and `reference.md` were decoded in memory. The documentation explicitly says the generated declarations are authoritative, that hooks run with **“no DOM and no Node”**, and that reload creates a fresh module environment. The plugin already contains those generated declarations:

`.claude-plugin/types/claude-code/index.d.ts` (path relative to plugin).

Relevant contracts in that file:

| Contract | Location | Consequence |
| --- | --- | --- |
| Native identity and name | `AgentInfo`, lines 125–163 | Child ID is identity; name is the SendMessage address, not a durable lookup key for reused names. |
| Spawn prompt and cwd | `AgentSpawnInput`, lines 240–319 | Capture the literal spawn prompt and effective cwd before the child starts. |
| Finished-child delivery | `SessionSendAddress`, lines 10824–10843 | **“A finished subagent is resumed from its transcript with the message, as the tool does.”** |
| Follow-up text | `SessionSendInput`, lines 10864–10900 | `session.send.text` is the message; `agentId` here identifies the **sender**, not the recipient. |
| Receiver delivery | `SessionReceiveInput`, lines 10650–10686 | `session.receive.text` is the body; its `agentId` identifies the receiving child. |
| Model-step input | `TurnStepInput`, lines 12620–12671 | Contains identity/model/count, not a prompt or transcript. Read the selected child's messages explicitly. |
| API conversation | `SessionMessagesApiArgs`/result, lines 10496–10558 | `$.session.messages({as:"api", agentId})` returns role/content blocks, or a denial; never silently substitute main context. |
| Working directory | `ProcessSpawnRequest`, lines 7625–7645 | `cwd` is supported; omitting it uses session cwd. |
| Resolve explicit cwd | `FsStat`, lines 4691–4723 | `$.fs.stat(path,{resolve:true}).realPath` is the host-supported absolute resolution; no Node path API needed. |

Also inspected the installed host's embedded resume implementation (`chunk-k0rsrfea.js`, export `resumeAgentBackground`). Its shared `zt` resume function loads the saved child transcript, preserves the child ID, builds a new user message `be` from follow-up `prompt:w` (including the delivery envelope), and supplies `promptMessages: h ? q : [...q,...Je,be]` to the resumed run. Thus the follow-up reaches **the selected child's API conversation**, not `agent.spawn.prompt` or a new field on `turn.step`. The proposed hook reads that conversation directly; it need not implement or intercept SendMessage delivery.

The defect is present in the inspected source. Generated types establish prior loading, not that an arbitrary currently running Claude process has loaded a particular revision. No live-session deployment/version claim is made.

## 2. Recommended behavior

**Start a fresh standalone Pi worker on each resumed native turn.** Keep the original task and cwd by child ID until session end/clear; supply the latest successful Pi final answer as a compact prior-work summary alongside the host's full current child conversation, which includes the new message. Explicitly tell the worker to answer the follow-up instead of replaying the original task.

Installed `pi --help` confirms:

```text
--continue, -c                 Continue previous session
--resume, -r                   Select a session to resume
--session <path|id>            Use specific session file or partial UUID
--session-id <id>              Use exact project session ID, creating it if missing
--no-session                   Don't save session (ephemeral)
--no-supervisor                Standalone worker; requires --no-session; no orchestration or Supervisor
```

Pi supports resuming persisted sessions, but **not the persisted standalone worker required by this plugin's present contract**. Removing `--no-session` conflicts with `--no-supervisor`; removing the latter changes the deployed no-orchestration boundary. Do not make that unrelated change. Do not use `--continue`: concurrent children share a cwd, so “most recent session” is not child identity. The explicit `--session-id` option can create a missing session and is not proof that an earlier session exists.

State needed for actual same-Pi-session continuation would be a persisted Pi transcript plus its exact session path/ID, original task, cwd, and native-child mapping. Current workers save none of that transcript. Recommended fresh-worker continuation needs only `{name, agentId, prompt, cwd, lastAnswer}` plus the current native transcript. `lastAnswer` is a summary, **not** the original Pi tool trace; native transcripts contain native answers but do not reconstruct Pi-internal tool calls.

Name reservations and retained child records have different lifetimes: release the reservation on completion, retain the ID record. ID-first binding lets an old child resume safely after another child reuses its name. Keep failed-spawn cleanup and session-end/clear cleanup. No automatic retry or Claude fallback.

Scope: finished-child continuation in the same loaded plugin environment. This minimal patch does not add cross-Claude-restart or plugin-hot-reload persistence: module Maps reset on reload, and `session.end` clears them. Such persistence would require a separate host `$.state`/`$.store` contract and migration policy; it is not claimed here.

## 3. Exact hooks/register.js proposal

Keep existing constants, `withHeartbeat`, `readWorkerEvent`, `interpretWorkerOutput`, `deleteTaskBindings`, reservation handling, `updateSpawnTaskBindings`, native registration, and `register` unchanged. Add the following two complete helpers:

```js
async function readTaskCwd($, event) {
	if (event.cwd === undefined) return $.session.cwd();
	const stat = await $.fs.stat(event.cwd, { resolve: true });
	if (stat.kind !== "dir" || !stat.realPath) {
		throw new Error(`native Sol cwd unavailable: ${event.cwd}`);
	}
	return stat.realPath;
}

function buildWorkerInput(task, messages) {
	const resumed = task.lastAnswer !== undefined;
	const instruction = resumed
		? WORKER_INSTRUCTION.replace(
				"Complete only the original task below using your configured tools. ",
				"Continue the original task by answering the newest follow-up request in the native conversation using your configured tools. " +
					"Treat the original task and previous answer as context; do not repeat completed work. ",
			)
		: WORKER_INSTRUCTION;
	const summary = resumed
		? "\n\nPrevious Pi final answer (prior-work summary, JSON string):\n" +
			JSON.stringify(task.lastAnswer)
		: "";
	return (
		instruction +
		"Original task (JSON string):\n" +
		JSON.stringify(task.prompt) +
		summary +
		"\n\nNative conversation (JSON; preserve roles and content blocks):\n" +
		JSON.stringify(messages)
	);
}
```

Replace these five complete functions:

```js
async function* delegateConversation($, event, task) {
	const messages = await $.session.messages({
		as: "api",
		agentId: event.agentId,
	});
	if (!Array.isArray(messages)) {
		throw new Error(`native conversation unavailable: ${messages.deny}`);
	}
	if (messages.length === 0) {
		throw new Error("empty native conversation");
	}
	const worker = $.process.spawn({
		argv: [
			"pi",
			"--provider",
			"openai-codex",
			"--model",
			MODEL,
			"--no-session",
			"--no-supervisor",
			"--mode",
			"json",
			"-p",
		],
		cwd: task.cwd,
		input: buildWorkerInput(task, messages),
	});
	const output = { pending: "", stderr: "", answer: "" };
	for await (const chunk of withHeartbeat($, worker)) {
		if (chunk === HEARTBEAT) {
			yield "wait\n";
			continue;
		}
		const { stream, text } = chunk;
		if (stream === "stderr") {
			output.stderr += text;
			continue;
		}
		const lines = (output.pending + text).split("\n");
		output.pending = lines.pop();
		for (const line of lines) {
			const progress = readWorkerEvent(line, output);
			if (progress) yield progress;
		}
	}
	readWorkerEvent(output.pending, output);
	const { code, signal } = await worker.result;
	return interpretWorkerOutput({ exitCode: code, signal, ...output });
}

function registerNativeTaskSpawn(on, tasksByName, tasksById) {
	on("agent.spawn", async ($, event, next) => {
		if (!isSelectedSol(event.subagentType, event.model)) return next(event);
		if (!event.name) return { deny: "Sol task requires a native Agent name" };
		let task = tasksByName.get(event.name);
		if (task?.prompt !== undefined)
			return { deny: `Sol task name collision: ${event.name}` };
		if (!task) {
			task = { name: event.name, prompt: undefined, agentId: undefined };
			tasksByName.set(event.name, task);
		}
		task.prompt = event.prompt;
		try {
			task.cwd = await readTaskCwd($, event);
			const result = await next(event);
			updateSpawnTaskBindings(result, task, tasksByName, tasksById);
			return result;
		} catch (error) {
			deleteTaskBindings(task, tasksByName, tasksById);
			return {
				deny: `Sol task spawn failed: ${error instanceof Error ? error.message : String(error)}`,
			};
		}
	});
}

function registerNativeTaskCleanup(on, tasksByName, tasksById) {
	on("turn.complete", { agentId: /^/ }, ($, event, next) => {
		const task = tasksById.get(event.agentId);
		if (task && tasksByName.get(task.name) === task) {
			tasksByName.delete(task.name);
		}
		return next(event);
	});
	on("session.end", { reason: /^/ }, ($, event, next) => {
		tasksByName.clear();
		tasksById.clear();
		return next(event);
	});
}

function bindNativeTask(agents, event, tasksByName, tasksById) {
	const agent = agents.find((row) => row.id === event.agentId);
	const task =
		agent?.type === "pi-delegate:sol" &&
		(tasksById.get(event.agentId) ?? tasksByName.get(agent.name));
	const hasOriginalTask = task && typeof task.prompt === "string";
	const hasBoundIdMismatch = task?.agentId && task.agentId !== event.agentId;
	if (!hasOriginalTask || hasBoundIdMismatch) {
		throw new Error("original task unavailable for selected native Sol child");
	}
	task.agentId = event.agentId;
	tasksById.set(event.agentId, task);
	return task;
}

function registerNativeSolDelegation(on, tasksByName, tasksById) {
	on("turn.step", async function* ($, event, next) {
		if (!event.agentId || event.model !== MODEL) {
			return yield* next(event);
		}
		let answer;
		let stopReason = "end_turn";
		let textIndex = 0;
		try {
			const agents = await $.agent.list();
			const task = bindNativeTask(agents, event, tasksByName, tasksById);
			const worker = delegateConversation($, event, task);
			let step = await worker.next();
			while (!step.done) {
				yield { kind: "text", index: 0, text: step.value };
				textIndex = 1;
				step = await worker.next();
			}
			answer = step.value;
			task.lastAnswer = answer;
		} catch (error) {
			const reason = error instanceof Error ? error.message : String(error);
			answer = `GPT delegation failed: ${reason}`;
			stopReason = "refusal";
		}
		yield { kind: "text", index: textIndex, text: answer };
		yield { kind: "stop", stopReason, usage: null };
		return {
			turnId: event.turnId,
			index: event.index,
			answer,
			toolUses: [],
			stopReason,
			usage: null,
		};
	});
}
```

The previous answer is updated only after an exit-0 worker with nonempty assistant text passes `interpretWorkerOutput`; failed workers cannot replace it with partial stdout. Early child completion still works: ID binding occurs inside the first step before spawn returns; completion removes only the name reservation, and the late spawn result does not resurrect it.

## 4. Tests/delegate.test.ts proposal

These are proposed fixtures, not executed workers. Preserve `assistantEvent`, `worker`, `collect`, `response`, and `rejectCore`.

### Required fixture/expectation adjustments

Add this line inside `nativeFixture`, before registering `agent.spawn`:

```ts
on("session.cwd", () => ({ value: "/work" }));
```

In the existing exact process-invocation expectation for “GPT substitutes final text using the complete selected child transcript via stdin”, add `cwd: "/work"` alongside `argv` and `input`. Fresh stdin remains byte-for-byte unchanged. No worker flags change.

Add this complete expected-input helper:

```ts
function resumedWorkerInput(
	prompt: string,
	previousAnswer: string,
	context: OpValueOf["session.messages"],
) {
	return (
		instruction.replace(
			"Complete only the original task below using your configured tools. ",
			"Continue the original task by answering the newest follow-up request in the native conversation using your configured tools. " +
				"Treat the original task and previous answer as context; do not repeat completed work. ",
		) +
		"Original task (JSON string):\n" +
		JSON.stringify(prompt) +
		"\n\nPrevious Pi final answer (prior-work summary, JSON string):\n" +
		JSON.stringify(previousAnswer) +
		"\n\nNative conversation (JSON; preserve roles and content blocks):\n" +
		JSON.stringify(context)
	);
}
```

### Finished child, SendMessage, repeated continuation, and reused name

Add this complete test. The delivery hook controls the external host boundary: it appends the new message to the child's transcript and starts a new turn on the same ID, as the documented finished-child resume path does. It does not spawn another native child for the old task.

```ts
test("finished Sol child resumes twice by id after its name is reused", async ($, on) => {
	rejectCore(on);
	const fixture = nativeFixture($, on);
	let context: OpValueOf["session.messages"] = [
		{ role: "user", content: [{ type: "text", text: task }] },
	];
	const requests: Args<"process.spawn">[] = [];
	const answers = ["Revenue is 42.", "Margin is 12.", "Both figures reconciled."];
	let launches = 0;
	on("session.messages", (_, event) => {
		expect(event.agentId).toBe("child-a");
		expect(event.as).toBe("api");
		return { value: context };
	});
	on("process.spawn", async function* (_, event) {
		requests.push(event);
		const answer = answers[launches++];
		if (answer === undefined) throw new Error("Unexpected worker launch");
		yield { stream: "stdout", text: assistantEvent(answer) };
		return { value: { code: 0, signal: null } };
	});
	on("turn.complete", (_, event) => ({ text: event.answer }));
	const complete = async (turnId: string, answer: string) => {
		await $.turn.complete({
			agentId: "child-a", turnId, answer, durationMs: 1,
			isAborted: false, reason: "answer",
		});
		if (!Array.isArray(context)) throw new Error("Expected API messages");
		context = [...context, {
			role: "assistant", content: [{ type: "text", text: answer }],
		}];
		const agent = fixture.agents[0];
		if (agent === undefined) throw new Error("Expected native child");
		agent.status = "completed";
	};
	await fixture.start(task, "reused-name");
	expect(await collect($.turn.step(child))).toEqual(response(child, answers[0]!));
	await complete(child.turnId, answers[0]!);
	await fixture.start("Different task", "reused-name");
	expect(fixture.agents).toHaveLength(2);

	let delivery = 0;
	on("session.send", async (_, event) => {
		expect(event.to).toBe("child-a");
		if (!Array.isArray(context)) throw new Error("Expected API messages");
		context = [...context, {
			role: "user", content: [{ type: "text", text: event.text }],
		}];
		const previousAnswer = answers[delivery]!;
		const answer = answers[++delivery]!;
		const step = { ...child, turnId: `resume-${delivery}`, index: 0 };
		expect(await collect($.turn.step(step))).toEqual(response(step, answer));
		expect(requests[delivery]?.input).toBe(
			resumedWorkerInput(task, previousAnswer, context),
		);
		expect(requests[delivery]?.cwd).toBe("/work");
		expect(requests[delivery]?.argv).toEqual(requests[0]?.argv);
		await complete(step.turnId, answer);
		return { isDelivered: true };
	});
	await $.session.send({ to: { agentId: "child-a" }, text: "Now calculate margin." });
	await $.session.send({ to: { agentId: "child-a" }, text: "Reconcile both figures." });
	expect(launches).toBe(3);
	expect(fixture.agents).toHaveLength(2);
	expect(requests[0]?.input).toBe(workerInput(task, [
		{ role: "user", content: [{ type: "text", text: task }] },
	]));
});
```

### Retained explicit cwd

Add this complete test; resolving the explicit relative cwd makes later parent-directory changes irrelevant.

```ts
test("resume keeps the resolved original child cwd", async ($, on) => {
	rejectCore(on);
	const fixture = nativeFixture($, on);
	let context: OpValueOf["session.messages"] = transcript;
	on("fs.stat", (_, event) => {
		expect(event.path).toBe("../child-work");
		expect(event.resolve).toBe(true);
		return { value: {
			kind: "dir", size: 0, mtimeMs: 0, isLink: false,
			realPath: "/original/child-work",
		} };
	});
	on("session.messages", () => ({ value: context }));
	let launches = 0;
	on("process.spawn", async function* (_, event) {
		expect(event.cwd).toBe("/original/child-work");
		return yield* worker(++launches === 1 ? "Initial result" : "Follow-up result")();
	});
	on("turn.complete", (_, event) => ({ text: event.answer }));
	await $.agent.spawn({
		tool_use_id: "cwd-call", prompt: task, description: "Read report",
		subagentType: "pi-delegate:sol",
		provider: { plugin: "pi-delegate", tier: "user" },
		parentModel: "sonnet", background: false, fork: false,
		name: "cwd-child", cwd: "../child-work",
	});
	await collect($.turn.step(child));
	await $.turn.complete({
		agentId: "child-a", turnId: child.turnId, answer: "Initial result",
		durationMs: 1, isAborted: false, reason: "answer",
	});
	context = [
		...transcript,
		{ role: "assistant", content: [{ type: "text", text: "Initial result" }] },
		{ role: "user", content: [{ type: "text", text: "Continue in the same directory." }] },
	];
	const resumed = { ...child, turnId: "cwd-resume", index: 0 };
	expect(await collect($.turn.step(resumed))).toEqual(response(resumed, "Follow-up result"));
	expect(launches).toBe(2);
	expect(fixture.agents).toHaveLength(1);
});
```

The fixture's parent cwd remains `/work`; both workers must use `/original/child-work`, not that default. A separate parent-cwd drift fixture can be added if desired, but this already asserts the effective original directory rather than implementation shape.

### Replace contradictory early-completion test

Replace “child starts and completes before native spawn returns with its original task” with:

```ts
test("child completes before spawn returns and remains resumable", async ($, on) => {
	rejectCore(on);
	let context: OpValueOf["session.messages"] = transcript;
	let launches = 0;
	on("session.messages", () => ({ value: context }));
	on("process.spawn", async function* (_, event) {
		const answer = ++launches === 1 ? "early child result" : "resumed result";
		expect(event.input).toBe(launches === 1
			? workerInput(task)
			: resumedWorkerInput(task, "early child result", context));
		return yield* worker(answer)();
	});
	on("turn.complete", (_, event) => ({ text: event.answer }));
	const fixture = nativeFixture($, on, undefined, async (agentId) => {
		if (agentId !== "child-a") return;
		const step = { ...child, agentId };
		expect(await collect($.turn.step(step))).toEqual(response(step, "early child result"));
		await $.turn.complete({
			agentId, turnId: step.turnId, answer: "early child result",
			durationMs: 1, isAborted: false, reason: "answer",
		});
	});
	await fixture.start(task, "early-child");
	expect((await fixture.start("new task", "early-child")).deny).toBeUndefined();
	context = [
		...transcript,
		{ role: "assistant", content: [{ type: "text", text: "early child result" }] },
		{ role: "user", content: [{ type: "text", text: "Check one more fact." }] },
	];
	const resumed = { ...child, turnId: "early-resume", index: 0 };
	expect(await collect($.turn.step(resumed))).toEqual(response(resumed, "resumed result"));
	expect(launches).toBe(2);
});
```

### Replace old combined cleanup test

Remove the loop asserting that both `turn.complete` and `session.end` erase the child. Completion/name reuse is now covered above. Replace it with:

```ts
test("session end removes retained Sol records and releases caller names", async ($, on) => {
	rejectCore(on);
	const fixture = nativeFixture($, on);
	on("session.messages", () => ({ value: transcript }));
	let launches = 0;
	on("process.spawn", async function* () {
		launches += 1;
		return yield* worker()();
	});
	on("turn.complete", (_, event) => ({ text: event.answer }));
	on("session.end", (_, event) => ({ sessionId: event.sessionId }));
	await fixture.start(task, "reusable");
	await collect($.turn.step(child));
	await $.turn.complete({
		agentId: "child-a", turnId: child.turnId, answer: "done",
		durationMs: 1, isAborted: false, reason: "answer",
	});
	await $.session.end({ reason: "clear", sessionId: "s", resume: { id: "s" } });
	const refused = await collect($.turn.step({ ...child, turnId: "after-clear" }));
	expect(refused.result.stopReason).toBe("refusal");
	expect(refused.result.answer).toContain("original task unavailable");
	expect(launches).toBe(1);
	expect((await fixture.start("new task", "reusable")).deny).toBeUndefined();
});
```

Keep reversed-order child isolation, identity-mismatch refusal, failed-spawn/tool cleanup, model passthrough, context denial/empty-context, worker failure/no retry, JSON streaming, and heartbeat tests. Their behavioral expectations remain valid except for the one added `cwd` field.

## 5. Exact documentation changes

### README.md

Append after the Native handoff paragraph:

> A finished Sol child can receive a follow-up through native SendMessage in the same loaded plugin session. Completion releases its active name reservation but retains the original task, resolved cwd, and latest successful Pi answer by native child ID until session end/clear. A continuation starts a new standalone Pi worker with the original task, previous-answer summary, and current selected-child transcript, including the follow-up. Completed work is context, not a task to replay. Reusing a name for another child does not change the old child's binding.

Append to Worker contract and deployment boundary:

> Workers remain ephemeral (`--no-session --no-supervisor`); continuation does not resume a persisted Pi session or preserve Pi-internal tool history. The host transcript and previous final answer provide continuity. The original cwd is passed explicitly on every launch. Retained bindings are module-local: plugin reload or Claude process restart is not supported by this continuation mechanism; no recovery fallback is attempted.

Do not rewrite historical proof as proof of this proposal. Add a status line stating that continuation regression tests and installed native smoke proof remain pending.

### docs/specs/native-gpt-agent.md

Replace the completed cleanup requirement with:

```markdown
- [ ] Release active task-name reservations on child completion; retain original task, resolved cwd, native child ID, and latest successful answer until session end/clear. Failed spawns release both reservations and child records.
- [ ] Continue a finished native Sol child by its existing child ID, independently of a newly spawned child reusing its name. Start a fresh standalone Pi worker with the original task, latest successful-answer summary, and current selected-child API conversation containing the new follow-up; do not replay completed work.
- [ ] Pass the original resolved cwd explicitly to initial and continued workers. Keep `--no-session --no-supervisor --mode json -p`; do not use implicit latest-session selection or enable orchestration to obtain persisted Pi continuation.
```

Mark these unchecked until implemented and verified. Keep existing streaming/heartbeat and explicit-failure requirements. Under Tests asserting this spec, describe the proposed new coverage without asserting a new passing count: finished-child SendMessage path, repeated follow-ups, ID isolation after name reuse, early completion before spawn return, original cwd, and session-end cleanup. Retain historical proof dates/counts as history, not current acceptance evidence.

Append to Out of scope:

```markdown
- Persisted same-Pi-session continuation: standalone mode requires ephemeral sessions.
- Restoring module-local child bindings after plugin reload or Claude process restart.
```

## 6. Verification plan — not run

After an authorized implementation, from any directory:

```text
claude plugin test /home/osso-test/AgentConfig/claude-plugins/pi-delegate
claude plugin validate /home/osso-test/AgentConfig/claude-plugins/pi-delegate
```

First add regression tests and observe current-source failure; then apply the proposal and require the delegation suite plus unchanged watchdog tests to pass. Check generated API types as part of the normal fixture/typecheck workflow; no speculative `agent.resume` or conversation-provider API is introduced.

Later, only with explicit model-run authorization, use a Claude session that loaded the changed plugin: spawn a bounded Sol task with a harmless contextual token; wait for actual native completion and Pi worker exit; send a follow-up addressed to that finished child's ID; require an answer using the earlier token plus new request. Repeat after another child reuses the old name and with an explicit different cwd. Observe one new ephemeral Pi worker per continuation, existing standalone flags, no Supervisor/orchestration traffic, and no Claude fallback. This installed-host check is required before claiming the real SendMessage integration fixed; controlled callback tests alone are not live proof.

**Risk if applied today:** the old failure should become a valid same-environment continuation, but the proposed code/tests have not been executed or validated. Continuity is limited to native transcript plus final-answer summary, not Pi's hidden tool trace. Reload/restart still loses bindings. No claim of shipped, passing, or deployed work.
