let card_generator_ip = "http://192.168.2.200";
let server_ip = "http://192.168.2.100";

function
show_message (text)
{
    window.dispatchEvent(
        new CustomEvent("show-message", { detail: text })
    );
}


function
show_products (products, is_locked = true)
{
    window.dispatchEvent(
        new CustomEvent("show-products", { detail: { text: products, isLocked: is_locked }})
    );
}


function
ask (message, labels, on_confirm)
{
    window.dispatchEvent(new CustomEvent("open-input", {
        detail: { message: message, labels: labels, callback: on_confirm }
    }));
}

function
card_handler()
{
    return {
        total_active_card: "-",
        total_lost_card: "-",
        
        init()
        {
            this.$watch("page", value => {
                 if (value === 'card') {
                     this.get_statistics();
                 }
             });
            if (this.page === "card") {
                this.get_statistics();
            }
        },
       
        async get_statistics()
        {
            let total_active_card = await fetch(server_ip + "/get_total_active_card");
            let total_lost_card = await fetch(server_ip + "/get_total_lost_card");
            
            this.total_active_card = total_active_card.text();
            this.total_lost_card = total_lost_card.text();
        },
        
        async generate_card()
        {
            let abort_controller = new AbortController();
            let timeout = setTimeout(() => abort_controller.abort(), 5000);
            
            
            try {
                let ping_response = await fetch(card_generator_ip + "/ping", {
                    signal: abort_controller.signal
                });

                clearTimeout(timeout);

                let text = await ping_response.text();
                
                if (text.trim() === "TAP") { 
                    show_message("TAP RFID");
                }

                try {
                    let response = await fetch(card_generator_ip);
                    let content_type = response.headers.get("Content-Type");

                    if (content_type && content_type.includes("application/octet-stream")) {
                        const buffer = await response.arrayBuffer();
                        const bytes = new Uint8Array(buffer);
                        
                        const uuid = Array.from(bytes).map(b => b.toString(16).padStart(2, "0"));
                        const uuid_str = [
                                uuid.slice(0, 4).join(""), 
                                uuid.slice(4, 6).join(""),
                                uuid.slice(6, 8).join(""),
                                uuid.slice(8, 10).join(""),
                                uuid.slice(10, 16).join("")
                            ];

                        show_message(`${uuid_str[0]}`);
                    } else {
                        const text = await response.text();
                        show_message(text);
                    }
                } catch(e) {
                    show_message(`FAILED TO GENERATE\n${e}`);
                } 
            } catch (e) {
                if (e.name === "AbortError") {
                    show_message("CARD GENERATOR TOOK TOO LONG TO RESPOND");
                } else {
                    show_message("UNEXPECTED ERROR OCCURRED");
                }
            }
        },

        async mark_as_lost() {
            ask("MARK AS LOST", {label1: "CARD UUID"}, async (data) => {
                try {
                    let uuid = data.val1;
                    let response = await fetch(server_ip + "/make-card-lost", {
                        method: "POST",
                        headers: {
                            "Content-Type": "text/plain",
                        },
                        body: uuid
                    });

                    let result = await response.text();
                    
                    if (result.trim() === "Success: UNLOST") {
                        show_message(`${uuid} MARKED AS UNLOST SUCCESS`);
                    } else if (result.trim() === "Success: LOST") {
                        show_message(`${uuid} MARKED AS LOST SUCCESS`);
                    } else if (result.trim() === "Not Found") {
                        show_message(`${uuid} NOT FOUND`);
                    } else if (result.trim() === "Invalid UUID") {
                        show_message("CARD UUID INVALID");
                    } else {
                        show_message("UNEXPECTED ERROR OCCURRED");
                    }
                } catch(e) {
                    show_message(e);
                }
            }); 
        }
    }
}

function
balance_handler()
{
    return {
        total_balance: "-",

        init()
        {
            this.$watch("page", value => {
                 if (value === 'balance') {
                     this.get_statistics();
                 }
             });
            if (this.page === "balance") {
                this.get_statistics();
            }
        },
        async get_statistics()
        {
            let response = await fetch(server_ip + "/total_balance");
            let text = await response.text();
            this.total_balance = text;
        },
        async add_balance() 
        {
            ask("ADD BALANCE", {label1: "CARD UUID", label2: "AMOUNT"}, async (data) => {
                let body_str = `${data.val1},${data.val2}`;
                let response = await fetch(server_ip + "/add_balance", {
                    method: "POST",
                    headers: { 
                        "Content-Type": "text/plain",
                    },
                    body: body_str 
                });
                let text = await response.text();

                if (text.trim() === "Invalid CARD UUID") {
                    show_message("INVALID CARD UUID");
                } else if (text.trim() === "Card UUID Not Found") {
                    show_message("CARD UUID NOT FOUND");
                } else if (text.trim() === "Failed") { 
                    show_message("UNEXPECTED ERROR OCCURRED");
                } else {
                    show_message(`${data.val1} UPDATED BALANCE: ₱${text}`);
                    this.get_total_balance();
                }
            }); 
        }
    }
}

function
product_handler()
{
    return {
        total_products: "-",
        
        init()
        {
            this.$watch("page", value => {
                 if (value === 'product') {
                     this.get_statistics();
                 }
             });
            if (this.page === "product") {
                this.get_statistics();
            }
        },
        async get_statistics()
        {
            let response = await fetch(server_ip + "/get_total_products");
            let text = await response.text();
            this.total_products = text;
        },
        async add_product()
        {
            ask("CREATE PRODUCT", {label1: "NAME", label2: "PRICE", label3: "INVENTORY"}, async (data) => {
                let body_str = `${data.val1},${data.val2},${data.val3}`;
                let response = await fetch(server_ip + "/add_product", {
                    method: "POST",
                    headers: {
                        "Content-Type": "text/plain"
                    },
                    body: body_str 
                });
                
                let text = await response.text();
                
                if (text.trim() === "Invalid Data") {
                    show_message("GIVEN DATA ARE INVALID!");
                } else if (text.trim() === "Failed" || text.trim() === "Cannot retrieve product ID") {
                    show_message("PRODUCT INVALID");
                } else {
                    show_message(`CREATED: ${data.val1}-${text}`);
                }

            });
        },
        async add_product_inventory()
        {
            ask("ADD PRODUCT INVENTORY", {label2: "PRODUCT ID", label3: "AMOUNT"}, async (data) => {
                let body_str = `${data.val2},${data.val3}`;

                let response = await fetch(server_ip + "/add_product_inventory", {
                    method: "POST",
                    headers: {
                        "Content-Type": "text/plain",
                    },
                    body: body_str
                });

                let text = await response.text();

                if (text.trim() === "Invalid Data") {
                    show_message("GIVEN DATA ARE INVALID");
                } else if (text.trim() === "Failed" || text.trim() === "Cannot retrieve updated inventory") {
                    show_message("PRODUCT INVALID");
                } else {
                    show_message(`UPDATED INVENTORY: ${text}`);
                }
            });
        },
    }
}

function
order_handler()
{
    return {
        not_delivered: "-",
        delivered: "-",
        init()
        {
            this.$watch("page", value => {
                 if (value === 'order') {
                     this.get_statistics();
                 }
             });
            if (this.page === "order") {
                this.get_statistics();
            }
        },
        async get_statistics()
        {
            let not_delivered = await fetch(server_ip + "/get_not_delivered");
            let delivered = await fetch(server_ip + "/get_delivered");
        
            this.not_delivered = await not_delivered.text(); 
            this.delivered = await delivered.text();
        },
        async deliver_order()
        {
            let scanner = window.__TAURI__.barcodeScanner;

            try {
                let permission = await scanner.requestPermissions();

                if (permission !== "granted") {
                    show_message("Please give camera permission");
                    return;
                }

                let result = await scanner.scan({
                    windowed: false,
                    formats: ["CODE_128"]
                });

                if (result && result.content) {
                    try {
                        let response = await fetch(server_ip + "/deliver_order", {
                            method: "POST",
                            headers: { "Content-Type": "text/plain" },
                            body: result.content
                        });

                    
                        let text = await response.text();
                    
                        if (response.ok) {
                            show_products(text.toUpperCase()); 
                            this.get_statistics();
                        } else {
                            show_message(text);
                        }
                    } catch(e) {
                        show_message(e);
                    }
                }
            } catch(e) {
                show_message("UNEXPECTED  ERROR");
            }

        }
    }
}
