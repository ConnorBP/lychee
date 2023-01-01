#include<SoftwareSerial.h>

SoftwareSerial serialOut(2,3);//rx tx

void setup() {
  while(!Serial) {;}// wait for connection to pc
  Serial.begin(9600);
  serialOut.begin(9600);
  Serial.println("connected to mouse");
}

void loop() {
  if (Serial.available()) {      // If anything comes in Serial (USB),
    serialOut.write(Serial.read());   // read it and send it out Serial1 (pins 0 & 1)
  }

  if (serialOut.available()) {     // If anything comes in Serial1 (pins 0 & 1)
    serialOut.write(serialOut.read());   // read it and send it out Serial (USB)
  }
}
