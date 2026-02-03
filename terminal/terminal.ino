#define BUZZER_PIN 13

// LCD Libraries
#include <Wire.h>
#include <LCDI2C_Multilingual.h>

LCDI2C_Generic lcd(0x27, 16, 2);

// RFIDRC522 Libraries
#include <SPI.h>
#include <MFRC522v2.h>
#include <MFRC522DriverSPI.h>
#include <MFRC522DriverPinSimple.h>
#include <MFRC522Debug.h>

#define RC522_SS_PIN 5
#define RC522_RST_PIN 21

MFRC522DriverPinSimple ss_pin(RC522_SS_PIN);
MFRC522DriverSPI driver{ss_pin};
MFRC522 mfrc522{driver};

// WiFi
#include <WiFi.h>
#include <ESPAsyncWebServer.h>
#include <LittleFS.h>

const char* station_ssid = "DEV";
const char* station_password = "devDevDev$123";

const char* access_point_ssid = "PUSIT";
const char* access_point_password = "";

AsyncWebServer server(80);

void
buzz(int duration)
{
    digitalWrite(BUZZER_PIN, HIGH);
    delay(duration);
    digitalWrite(BUZZER_PIN, LOW);
}

void
lcd_print(char* t)
{
    lcd.print(t);
    delay(500);
    lcd.clear();
}

void
setup_routes()
{
    server.on("/", HTTP_GET, [](AsyncWebServerRequest *request) {
        request -> redirect("/index.html");
    });
}

void
setup()
{
    pinMode(RC522_RST_PIN, OUTPUT);
    pinMode(BUZZER_PIN, OUTPUT);
    digitalWrite(RC522_RST_PIN, HIGH);
    
    // LCD Setup
    Wire.begin(25, 26);
    lcd.init();
    lcd.backlight();
    lcd_print("/ LCD");

    // RFIDRC522 Setup
    SPI.begin(18, 19, 23, RC522_SS_PIN);
    lcd_print("/ RFID");

    // WiFi
    WiFi.mode(WIFI_AP_STA);
    WiFi.begin(station_ssid, station_password);
    WiFi.softAP(access_point_ssid, access_point_password);

    if(!LittleFS.begin(true)) {
        lcd_print("X MOUNT DATA");
        return;
    }
    
    lcd_print("/ DATA");

    server.serveStatic("/", LittleFS, "/");

    server.begin();

    lcd_print("/ WEB");
    lcd_print("/ STATION");
    lcd.print(WiFi.softAPIP());
}

void
loop()
{
}
