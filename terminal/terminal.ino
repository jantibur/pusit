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
MFRC522 mfrc522{driver};

// WiFi
#include <WiFi.h>
#include <ESPAsyncWebServer.h>
#include <LittleFS.h>
#include <HTTPClient.h>


const char* station_ssid = "SQUID";
const char* station_password = "squid1234";

const char* access_point_ssid = "PUSIT";
const char* access_point_password = "";

AsyncWebServer server(80);

void
setup()
{
    Serial.begin(115200);
    // RFIDRC522 Setup
    SPI.begin(18, 19, 23, RC522_SS_PIN);

    // WiFi
    WiFi.mode(WIFI_AP_STA);
    WiFi.begin(station_ssid, station_password);
    WiFi.softAP(access_point_ssid, access_point_password);

    if(!LittleFS.begin(true)) {
        Serial.println("Cannot initialize LittleFS");
        return;
    }    

    server.serveStatic("/", LittleFS, "/");

    server.on("/get_products", HTTP_GET, [](AsyncWebServerRequest *request) {
        HTTPClient http;

        http.begin(String(SERVER_IP) + "get_products");

        int http_code = http.GET();

        String products;

        if (http_code == 200) {
            products = http.getString(); 
        }

        http.end();

        request->send(200, "plain/text", products);
    });


    server.begin();
}

void
loop()
{
}
