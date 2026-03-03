let server_ip = "http://192.168.2.100";
let reader_ip = "http://192.168.2.101";

function show_message(text, isLocked = false) {
    window.dispatchEvent(
        new CustomEvent("show-message", { 
            detail: { text: text, isLocked: isLocked } 
        })
    );
}

function
handle_products()
{
	return {
		products: [],
		cart: [],
		cart_price: 0,
		view: "order",
		search: '',
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
					if(parseInt(product[3]) > 0) {
						products.push(product);
					}
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
            let abort_controller = new AbortController();
            let id = setTimeout(() => abort_controller.abort(), 12000);

            let product_ids = [];
            
            for (let i = 0; i < this.cart.length; i++) {
				product_ids.push(`${this.cart[i][0]}`);
			}
			
			let body_str = product_ids.join();
            
            try {
                let init = await fetch(reader_ip + `/create_order?ordered_products=${encodeURIComponent(body_str)}`, {
					signal: abort_controller.signal,
                });

                let init_status = await init.text();
    
                if (init_status.trim() === "67") {
                    show_message("TAP YOUR CARD");
                    
                    let is_card_read = false;

                    while (!is_card_read) {
                        let update = await fetch(reader_ip + "/create_order_status");
                        let updated_status = await update.text();

                        if (updated_status.trim() ===  "NOT WAITING" || updated_status.trim() === "WAITING") {
                            await new Promise(r => setTimeout(r, 500));
                        } else if (updated_status.trim().length == 19){
                            window.onbeforeunload = function() { return "SAVE YOUR GENERATED BARCODE!" }; 

                            show_message("GENERATED BARCODE", true);

                            JsBarcode("#order-reference", updated_status.trim(), {
                                width: 2,
                                height: 200,
                                displayValue: true
                            });
                            is_card_read = true;
                        } else {
                            window.onbeforeunload = function() { return "PLEASE PUT ENOUGH BALANCE TO YOUR CARD" }; 
                            show_message(updated_status, true);
                            is_card_read = true;
                        }
                    }
                }

            } catch(e) {
                if (e.name === "AbortError") {
                    show_message("TIMEOUT");
                } else {
                    show_message(e);
                }
            } finally {
                clearTimeout(id);
            }
		},
		
		get filtered_products()
		{
			if (this.search.trim() === "") {
				return this.products;    
			} else {
				return this.products.filter(product => {
					return product[1].toLowerCase().includes(this.search.toLowerCase());
				});
			}
		},
	}
}
