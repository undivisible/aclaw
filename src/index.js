import { Container } from "cloudflare:container";

export class UnthinkclawContainer extends Container {
	defaultPort = 8080;
	sleepAfter = "10m";

	override onStart() {
		console.log("unthinkclaw container started");
	}

	override onStop() {
		console.log("unthinkclaw container stopped");
	}

	override onError(error) {
		console.error("unthinkclaw error:", error);
	}
}

export default {
	async fetch(request, env) {
		const url = new URL(request.url);

		if (url.pathname === "/health") {
			return new Response("ok");
		}

		const id = env.UNTHINKCLAW.idFromName("default");
		const stub = env.UNTHINKCLAW.get(id);
		return stub.fetch(request);
	},
};
