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
                     this.get_total_balance();
                 }
             });
            if (this.page === "balance") {
                this.get_total_balance();
            }
        },
        async get_total_balance()
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
