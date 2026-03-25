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

// ── Anthropic helper ──────────────────────────────────────────────────────────

async function callAnthropic(apiKey, userText) {
	// OAuth tokens (sk-ant-oat*) use Bearer auth; direct keys use x-api-key
	const isOAuth = apiKey.startsWith("sk-ant-oat");
	const authHeaders = isOAuth
		? { Authorization: `Bearer ${apiKey}` }
		: { "x-api-key": apiKey };

	const res = await fetch("https://api.anthropic.com/v1/messages", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
			"anthropic-version": "2023-06-01",
			...authHeaders,
		},
		body: JSON.stringify({
			model: "claude-sonnet-4-6",
			max_tokens: 2048,
			system: "You are a helpful AI assistant. Be concise and clear.",
			messages: [{ role: "user", content: userText }],
		}),
	});

	const data = await res.json();
	if (!res.ok) {
		throw new Error(data.error?.message ?? `Anthropic ${res.status}`);
	}
	return data.content?.[0]?.text ?? "(no response)";
}

// ── Bot logic ─────────────────────────────────────────────────────────────────

async function handleUpdate(env, update) {
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

	const reply = await callAnthropic(env.ANTHROPIC_API_KEY, text);
	await sendMessage(env.TELEGRAM_BOT_TOKEN, chatId, reply);
}

// ── Worker entrypoint ─────────────────────────────────────────────────────────

export default {
	async fetch(request, env, ctx) {
		const url = new URL(request.url);

		if (url.pathname === "/health") {
			return new Response("ok");
		}

		if (url.pathname === "/webhook" && request.method === "POST") {
			const update = await request.json().catch(() => null);
			if (update) {
				// Process in background — return 200 to Telegram immediately
				ctx.waitUntil(
					handleUpdate(env, update).catch((err) =>
						console.error("bot error:", err.message)
					)
				);
			}
			return new Response("ok", { status: 200 });
		}

		// All other routes → container (MCP server)
		const id = env.UNTHINKCLAW.idFromName("default");
		const stub = env.UNTHINKCLAW.get(id);
		return stub.fetch(request);
	},
};
