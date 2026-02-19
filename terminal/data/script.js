let server_ip = "http://192.168.1.227";

function
handle_products()
{
	return {
		products: [],
		cart: [],
		cart_price: 0,
		view: "order",
		
		async get_products()
		{
			let response = await fetch(server_ip + "/get_products");
			let text = await response.text();

			if (text.trim() === "Failed") {
				this.product_string = "Fetching products failed...";	
			} else {
				let products_data = text.split(",");
				products_data.splice(products_data.length - 1);
				
				let products = [];
				
				for (let i = 0; i < products_data.length; i++) {
					let product = products_data[i].split("|");
					products.push(product);
				}
				this.products = products;
			}
		},
		
		add_to_cart(product)
		{
			this.cart.push(product);
			this.get_cart_price();
		},

		async get_cart_price()
		{
			let price = 0;

			for (let i = 0; i < this.cart.length; i++) {
				price += parseInt(this.cart[i][2]);
			}
			
			this.cart_price = price;
		},
		
		async process_order()
		{
			let product_ids = [];

			for (let i = 0; i < this.cart.length; i++) {
				product_ids.push(`${this.cart[i][0]}`);
			}
			
			let body_str = product_ids.join();
		
		}
	}
}
