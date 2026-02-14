#include <SPI.h>
#include <MFRC522v2.h>
#include <MFRC522Constants.h>
#include <MFRC522DriverSPI.h>
#include <MFRC522DriverPinSimple.h>
#include <MFRC522Debug.h>

#define RFID_SS_PIN 2

MFRC522DriverPinSimple ss_pin(RFID_SS_PIN);
MFRC522DriverSPI driver{ss_pin};

MFRC522 rfid{driver};

#include <Ethernet.h>

#define ETHERNET_SS_PIN 5

byte mac[] = { 0x02, 0x12, 0x34, 0x56, 0x78, 0x9A };

/* Change the subnet depending on router setup */
IPAddress ip(192, 168, 2, 200);

/* Change the subnet depending on the router setup */
IPAddress root_server(192, 168, 2, 100);

EthernetServer server(80);

MFRC522::MIFARE_Key key;
MFRC522::StatusCode status;


EthernetClient current_client;
bool requested = false;

EthernetClient post_client;

void
setup() 
{
    /* For debugging purposes only */
    Serial.begin(115200);
    delay(1000);

    for (byte i = 0; i < 6; i++) {
        key.keyByte[i] = 0xFF;
    }

    delay(1000);

    rfid.PCD_Init();
    delay(1000);
    

    Ethernet.init(ETHERNET_SS_PIN);
    
    Ethernet.begin(mac, ip);


    if (Ethernet.hardwareStatus() == EthernetNoHardware) {
        Serial.println("Ethernet shield was not found.");
    }

    if (Ethernet.linkStatus() == LinkOFF) {
        Serial.println("Ethernet cable is not connected.");
    }

    server.begin();
    Serial.print("Running on: ");
    Serial.print(Ethernet.localIP());
    Serial.println();
}

void
send_uuid(EthernetClient &client, byte (&uuid)[16])
{
    client.println("HTTP/1.1 200 OK");
    client.println("Access-Control-Allow-Origin: *");
    client.println("Content-Type: application/octet-stream");
    client.print("Content-Length: ");
    client.println(16);
    client.println("Connection: close");
    client.println();
    client.write(uuid, 16);
}

void
send_rfid_uid_data(EthernetClient &client, MFRC522 &rfid)
{
    if (client.connect(root_server, 80)) {    
        client.println("POST /generate-card HTTP/1.1");
        client.print("Host: ");
        client.println(root_server);
        client.println("User-Agent: curl/8.0");
        client.println("Accept: */*"); 
        client.println("Content-Type: application/octet-stream");
        client.print("Content-Length: ");
        client.println(4); 
        client.println();
        client.write(rfid.uid.uidByte, 4);
    }

}

void
dump_byte_array(byte *buffer, byte buffer_size)
{
    for (byte i = 0; i < buffer_size; i++) {
        Serial.print(buffer[i] < 0x10 ? "0" : " ");
        Serial.print(buffer[i], HEX);
    }
}

bool
is_byte_array_empty(byte *byte_array, byte array_size)
{
    int is_empty = true;
    for (byte i = 0; i < array_size; i++) {
        if (byte_array[i] != 0) {
            is_empty = false;
            break;
        }
    }
    return is_empty;
}


bool
is_card_supported(MFRC522 &rfid)
{
    MFRC522::PICC_Type picc_type = rfid.PICC_GetType(rfid.uid.sak);

    if (picc_type != MFRC522Constants::PICC_TYPE_MIFARE_MINI && picc_type != MFRC522Constants::PICC_TYPE_MIFARE_1K && picc_type == MFRC522Constants::PICC_TYPE_MIFARE_4K) { 
        return false;
    }
    return true;
}

void
read_server_uid_block(MFRC522 &rfid, EthernetClient &client)
{
    byte sector = 1;
    byte block_address = 4;
    byte last_block_address = 7;

    byte buffer[18];
    byte size = sizeof(buffer);

    status = rfid.PCD_Authenticate(
        MFRC522Constants::PICC_CMD_MF_AUTH_KEY_A, block_address, &key, &(rfid.uid)
    );

    if (status != MFRC522Constants::STATUS_OK) {
        rfid.PICC_HaltA();
        rfid.PCD_StopCrypto1();
        close_request_session(rfid, client, "FAILED: AUTH KEY A");
        requested = false;
        current_client.stop();
    } 

    status = rfid.MIFARE_Read(block_address, buffer, &size);
 
    if (status != MFRC522Constants::STATUS_OK) {
        rfid.PICC_HaltA();
        rfid.PCD_StopCrypto1();
        close_request_session(rfid, client, "FAILED: READ");
        requested = false;
        current_client.stop();
    }

    dump_byte_array(buffer, 16);
}

void
write_server_uid_block(MFRC522 &rfid, EthernetClient &client, byte (&data_block)[16])
{ 
    byte sector = 1;
    byte block_address = 4;
    byte last_block_address = 7;

    status = rfid.PCD_Authenticate(
        MFRC522Constants::PICC_CMD_MF_AUTH_KEY_B, last_block_address, &key, &(rfid.uid)
    );

    if (status != MFRC522Constants::STATUS_OK) {
        rfid.PICC_HaltA();
        rfid.PCD_StopCrypto1();
        close_request_session(rfid, client, "FAILED: AUTH KEY B");
        requested = false;
        current_client.stop();
    } 

    status = rfid.MIFARE_Write(block_address, data_block, 16);
 
    if (status != MFRC522Constants::STATUS_OK) {
        rfid.PICC_HaltA();
        rfid.PCD_StopCrypto1();
        close_request_session(rfid, client, "FAILED: WRITE");
        requested = false;
        current_client.stop();
    }
}

void
close_request_session(MFRC522 &rfid,  EthernetClient &client, String message)
{

    client.println("HTTP/1.1 200 OK");
    client.println("Content-Type: text/plain"); 
    client.println("Access-Control-Allow-Origin: *");
    client.println("Connection: close");
    client.println();
    client.println(message);
    rfid.PICC_HaltA();
    rfid.PCD_StopCrypto1();
    requested = false;
    client.stop();
}

void 
loop() 
{ 
    if (!requested) {  
        EthernetClient client = server.available();
        if (client) { 
            requested = true;
            current_client = client;
        }
    }

    if (requested) { 
        if (rfid.PICC_IsNewCardPresent() && rfid.PICC_ReadCardSerial()) {        

            bool supported = is_card_supported(rfid);
            
            if (supported) { 
                Serial.print("BEFORE: ");
                read_server_uid_block(rfid, current_client);
                
                send_rfid_uid_data(post_client, rfid);

                
                byte response[32];


                byte server_uid[16];
                byte uuid[16];

                while (post_client.connected()) {
                    if (post_client.available()) {
                        if (post_client.find("\r\n\r\n")) {
                            post_client.read(response, 32);
                        }
                        break;
                    }
                }
                
                post_client.stop();

                bool is_response_empty = is_byte_array_empty(response, 32);
            
                if (is_response_empty) {
                    Serial.println();
                    Serial.println("CARD INVALID");
                    close_request_session(rfid, current_client, "FAILED: CARD INVALID"); 
                    return;
                }

                memcpy(server_uid, &response[0], 16);
                memcpy(uuid, &response[16], 16);  
                
                write_server_uid_block(rfid, current_client, server_uid);
                Serial.println();
                Serial.print("AFTER: ");
                read_server_uid_block(rfid, current_client);
                Serial.println();
                Serial.print("UUID: ");
                dump_byte_array(uuid, 16);
                Serial.println();        
                send_uuid(current_client, uuid);
            }
        }
    }
}
