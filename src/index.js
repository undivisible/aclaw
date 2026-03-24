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

export default {
	async fetch(request, env) {
		const url = new URL(request.url);

		if (url.pathname === "/health") {
			return new Response("ok");
		}

		// Acknowledge Telegram webhooks without waking the container
		if (url.pathname === "/webhook") {
			return new Response("ok", { status: 200 });
		}

		const id = env.UNTHINKCLAW.idFromName("default");
		const stub = env.UNTHINKCLAW.get(id);
		return stub.fetch(request);
	},
};
