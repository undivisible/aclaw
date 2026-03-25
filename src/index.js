import { Container } from "@cloudflare/containers";
import { env } from "cloudflare:workers";

export class UnthinkclawContainer extends Container {
	defaultPort = 8080;
	sleepAfter = "10m";

	envVars = {
		ANTHROPIC_API_KEY: env.ANTHROPIC_API_KEY,
		TELEGRAM_BOT_TOKEN: env.TELEGRAM_BOT_TOKEN,
	};

	onStart() {
		console.log("unthinkclaw container started");
	}

	onStop() {
		console.log("unthinkclaw container stopped");
	}

	onError(error) {
		console.error("unthinkclaw error:", error);
	}
}

// ── Telegram helpers ───────────────────────────────────────────────────────────

async function tgCall(token, method, body) {
	const res = await fetch(`https://api.telegram.org/bot${token}/${method}`, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(body),
	});
	return res.json();
}

// ── Agent call via container ───────────────────────────────────────────────────
// The container handles all Telegram messaging (⏳ draft, tool progress, final response).
// We just fire-and-await; the container returns the final text (may be empty if already sent).

async function callAgent(containerStub, chatId, text, telegramToken) {
	const res = await containerStub.fetch(
		new Request("http://container/chat", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				text,
				chat_id: String(chatId),
				telegram_token: telegramToken,
			}),
		})
	);
	if (!res.ok) {
		const body = await res.text().catch(() => res.status.toString());
		throw new Error(`Agent error ${res.status}: ${body}`);
	}
	const data = await res.json();
	return data.text ?? "";
}

// ── Bot logic ─────────────────────────────────────────────────────────────────

async function handleUpdate(env, update, containerStub) {
	const msg = update?.message ?? update?.edited_message;
	if (!msg) return;

	const chatId = msg.chat?.id;
	const text = msg.text;
	if (!chatId || !text) return;

	const token = env.TELEGRAM_BOT_TOKEN;

	// Send initial typing — container will send ⏳ on first tool call
	tgCall(token, "sendChatAction", { chat_id: chatId, action: "typing" });

	try {
		// Container handles all Telegram messages (draft → progress → finalize).
		// If text comes back non-empty, it means draft mode wasn't used (no tools called).
		const reply = await callAgent(containerStub, chatId, text, token);
		if (reply) {
			// Direct/conversational response — container didn't send it, we do
			await tgCall(token, "sendMessage", {
				chat_id: chatId,
				text: reply,
			});
		}
	} catch (err) {
		await tgCall(token, "sendMessage", {
			chat_id: chatId,
			text: `❌ ${err.message.slice(0, 300)}`,
		});
	}
}

// ── Worker entrypoint ─────────────────────────────────────────────────────────

export default {
	async fetch(request, env, ctx) {
		const url = new URL(request.url);

		if (url.pathname === "/health") {
			return new Response("ok");
		}

		const id = env.UNTHINKCLAW.idFromName("default");
		const stub = env.UNTHINKCLAW.get(id);

		if (url.pathname === "/webhook" && request.method === "POST") {
			const update = await request.json().catch(() => null);
			if (update) {
				ctx.waitUntil(
					handleUpdate(env, update, stub).catch((err) =>
						console.error("bot error:", err.message)
					)
				);
			}
			return new Response("ok", { status: 200 });
		}

		return stub.fetch(request);
	},
};
