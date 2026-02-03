let server_addr = 'http://192.168.1.227:3000'

function get_messages()
{
	return {
		messages: {},

		async get_messages() {
			const res = await fetch(server_addr);
			this.messages = await res.json();
			console.log(this.messages);
		}
	}
}
