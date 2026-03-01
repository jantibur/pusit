#define SERVER_IP "http://192.168.2.100"

// RFIDRC522 Libraries
#include <SPI.h>
#include <MFRC522v2.h>
#include <MFRC522DriverSPI.h>
#include <MFRC522DriverPinSimple.h>
#include <MFRC522Debug.h>

#define RC522_SS_PIN 5

MFRC522DriverPinSimple ss_pin(RC522_SS_PIN);
MFRC522DriverSPI driver{ss_pin};
MFRC522 rfid{driver};

// WiFi
#include <WiFi.h>
#include <ESPAsyncWebServer.h>
#include <HTTPClient.h>

const char* station_ssid = "SQUID";
const char* station_password = "squid1234";

AsyncWebServer server(80);

MFRC522::MIFARE_Key key;
MFRC522::StatusCode status;

String ordered_products = "";
String reader_status = "NOT WAITING";
bool is_waiting_for_card = false;
unsigned long order_request_time = 0;

void
setup()
{
    Serial.begin(115200);

    for (byte i = 0; i < 6; i++) {
        key.keyByte[i] = 0xFF;
    }

    // RFIDRC522 Setup
    SPI.begin(18, 19, 23, RC522_SS_PIN);

    rfid.PCD_Init();
    WiFi.mode(WIFI_STA);
    WiFi.begin(station_ssid, station_password);

    DefaultHeaders::Instance().addHeader("Access-Control-Allow-Origin", "*");
    DefaultHeaders::Instance().addHeader("Access-Control-Allow-Methods", "GET, POST");
    DefaultHeaders::Instance().addHeader("Access-Control-Allow-Headers", "Content-Type");

    server.on("/create_order", HTTP_GET, [](AsyncWebServerRequest *request) {
        if (is_waiting_for_card) {
            request->send(200, "text/plain", "Busy");
            return;
        }

        if (request->hasParam("ordered_products")) {
            ordered_products = request->getParam("ordered_products")->value(); 
        }

        reader_status = "NOT WAITING";
        is_waiting_for_card = true;
        order_request_time = millis();
        request->send(200, "text/plain", "67");
    });

    server.on("/create_order_status", HTTP_GET, [](AsyncWebServerRequest *request) {
        request->send(200, "text/plain", reader_status);
        if (reader_status != "WAITING") {
            reader_status = "WAITING";
        }
    });


    server.begin();
}


void
loop()
{
    if (is_waiting_for_card) {

        if(millis() - order_request_time > 5000) {
            reader_status = "READ CARD TIMEOUT";
            is_waiting_for_card = false;
            return;
        }

        if (rfid.PICC_IsNewCardPresent() && rfid.PICC_ReadCardSerial()) {
                HTTPClient http;
                byte uid[4];
                byte uid_size = sizeof(uid);
                memcpy(uid, rfid.uid.uidByte, uid_size); 
               
                byte sector = 1;
                byte block_address = 4;
                byte last_block_address = 7;

                byte server_uid[18];
                byte server_uid_size = sizeof(server_uid);

                status = rfid.PCD_Authenticate(
                    MFRC522Constants::PICC_CMD_MF_AUTH_KEY_A, block_address,  &key, &(rfid.uid)
                );  

                if (status != MFRC522Constants::STATUS_OK) {
                    rfid.PICC_HaltA();
                    rfid.PCD_StopCrypto1();
                    reader_status = "CANNOT AUTHENTICATE";
                    is_waiting_for_card = false;
                }
                
                status = rfid.MIFARE_Read(block_address, server_uid, &server_uid_size);

                if (status != MFRC522Constants::STATUS_OK) {
                    rfid.PICC_HaltA();
                    rfid.PCD_StopCrypto1();
                    reader_status = "CANNOT READ";
                    is_waiting_for_card= false;
                }


                int ordered_products_len = ordered_products.length();
                
                byte data[22 + ordered_products_len];

                memcpy(data, uid, uid_size);
                memcpy(data + uid_size, server_uid, server_uid_size);
                memcpy(data + 22, ordered_products.c_str(), ordered_products_len);

                http.begin(String(SERVER_IP) + "/create_order");
                
                http.addHeader("Content-Type", "application/octet-stream");


                // This will return the reference number soon
                int http_response_code = http.POST(data, sizeof(data));

                if (http_response_code == 200) {
                    reader_status = http.getString();
                } else if (http_response_code == 403 || http_response_code == 401) {
                    reader_status = "CARD INVALID";
                } else if (http_response_code == 402) {
                    reader_status = http.getString();
                } else {
                    reader_status = http.getString();
                }

                rfid.PICC_HaltA();
                rfid.PCD_StopCrypto1();
                is_waiting_for_card = false;
                http.end();                
        }
    }

}
