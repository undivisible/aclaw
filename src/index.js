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

// ── Telegram helpers ──────────────────────────────────────────────────────────

const TG_MAX = 4000;

async function tgCall(token, method, body) {
	const res = await fetch(`https://api.telegram.org/bot${token}/${method}`, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(body),
	});
	return res.json();
}

async function sendMessage(token, chatId, text) {
	const chunks = [];
	for (let i = 0; i < text.length; i += TG_MAX) {
		chunks.push(text.slice(i, i + TG_MAX));
	}
	for (const chunk of chunks) {
		const res = await tgCall(token, "sendMessage", {
			chat_id: chatId,
			text: chunk,
			parse_mode: "Markdown",
		});
		// Fallback to plain text if Markdown parse fails
		if (!res.ok) {
			await tgCall(token, "sendMessage", { chat_id: chatId, text: chunk });
		}
	}
}

// ── Agent call via container ───────────────────────────────────────────────────

async function callAgent(containerStub, chatId, text) {
	const res = await containerStub.fetch(
		new Request("http://container/chat", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ text, chat_id: String(chatId) }),
		})
	);
	if (!res.ok) {
		const body = await res.text().catch(() => res.status.toString());
		throw new Error(`Agent error ${res.status}: ${body}`);
	}
	const data = await res.json();
	return data.text ?? "(no response)";
}

// ── Bot logic ─────────────────────────────────────────────────────────────────

async function handleUpdate(env, update, containerStub) {
	const msg = update?.message ?? update?.edited_message;
	if (!msg) return;

	const chatId = msg.chat?.id;
	const text = msg.text;
	if (!chatId || !text) return;

	// Typing indicator — fire and forget
	tgCall(env.TELEGRAM_BOT_TOKEN, "sendChatAction", {
		chat_id: chatId,
		action: "typing",
	});

	const reply = await callAgent(containerStub, chatId, text);
	await sendMessage(env.TELEGRAM_BOT_TOKEN, chatId, reply);
}

// ── Worker entrypoint ─────────────────────────────────────────────────────────

export default {
	async fetch(request, env, ctx) {
		const url = new URL(request.url);

		if (url.pathname === "/health") {
			return new Response("ok");
		}

		// Get container stub (shared for both webhook and MCP routes)
		const id = env.UNTHINKCLAW.idFromName("default");
		const stub = env.UNTHINKCLAW.get(id);

		if (url.pathname === "/webhook" && request.method === "POST") {
			const update = await request.json().catch(() => null);
			if (update) {
				// Process in background — return 200 to Telegram immediately
				ctx.waitUntil(
					handleUpdate(env, update, stub).catch((err) =>
						console.error("bot error:", err.message)
					)
				);
			}
			return new Response("ok", { status: 200 });
		}

		// All other routes → container (MCP server)
		return stub.fetch(request);
	},
};
