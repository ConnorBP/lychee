#include <BleMouse.h>
// lib: https://github.com/T-vK/ESP32-BLE-Mouse

// Some Bs Name for the mouse. Seems legitish
BleMouse bleMouse("Bluetooth(R) Mouse", "Microsoft (R)", 100);
String inputString = "";         // a String to hold incoming data
bool stringComplete = false;  // whether the string is complete

// when zero we can click again. If set to 1 it will increment until 1000 then go back to zero
// while not zero no clicks
int cooldown = 0;

void setup() {
  // put your setup code here, to run once:
  Serial.begin(115200);
  Serial.println("starting ble work!");
  // reserve 200 bytes for the inputString:
  inputString.reserve(200);
  bleMouse.begin();
}

bool prefix(const char *pre, const char *str)
{
    return strncmp(pre, str, strlen(pre)) == 0;
}

int get_int(const char *str, int startByte)
{
    int i = (int) ((str[startByte] << 24) | (str[startByte+1] << 16) | (str[startByte+2] << 8) | (str[startByte+3]));
    return i;
}

void loop() {

  // increment cooldown while greater than zero
  if(cooldown > 0) {
    cooldown++;
  }
  // once we reach end of cooldown, reset.
  if(cooldown >= 1000) {
    cooldown = 0;
  }
  
  if(bleMouse.isConnected() && stringComplete) {
    //Serial.println(inputString);
    if(inputString == "m0") {
      // mouse 0 (left click)
      if(cooldown == 0) {
        bleMouse.click(MOUSE_LEFT);
        cooldown = 1;
      }
    } else if(inputString == "m1") {
      // mouse 1 (right click)
      bleMouse.click(MOUSE_RIGHT);
    } else if(inputString == "ju") {
      // jump
      bleMouse.click(MOUSE_MIDDLE);
    } else if(inputString == "sd") {
      // scroll down
      bleMouse.move(0,0,-1);
    } else if(inputString == "su") {
      // scroll up
      bleMouse.move(0,0,1);
    } else if(prefix("mv", inputString.c_str())) { // move mouse x
      // get a 4 byte float from the string starting at the 3rd char
      int x = get_int(inputString.c_str(), 2);
      int y = get_int(inputString.c_str(), 6);
      bleMouse.move(x,y);
      Serial.print("move x ");
      Serial.println(x);
      Serial.print("move y");
      Serial.println(y);
    }
    
    inputString = "";
    stringComplete = false;
  }
}

/*
  SerialEvent occurs whenever a new data comes in the hardware serial RX. This
  routine is run between each time loop() runs, so using delay inside loop can
  delay response. Multiple bytes of data may be available.
*/
void serialEvent() {
  while (Serial.available()) {
    // get the new byte:
    char inChar = (char)Serial.read();
    // if the incoming character is a newline, set a flag so the main loop can
    // do something about it:
    if (inChar == '\n') {
      stringComplete = true;
    } else {
      // add it to the inputString:
      inputString += inChar;
    }
  }
}
