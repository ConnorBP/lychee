// BLE minion by segfault

#include <BleMouse.h>
// lib: https://github.com/T-vK/ESP32-BLE-Mouse

// Some Bs Name for the mouse. Seems legitish
BleMouse bleMouse("Bluetooth(R) Mouse", "Microsoft (R)", 100);

//
// Serial Com Settings
//

const char startOfNumberDelimiter = '<';
const char endOfNumberDelimiter = '>';


// Count how many number args we take in
int argc = 0;
// max 4 args
int args[4] = {0,0,0,0};

String inputString = "";         // a String to hold incoming data
bool stringComplete = false;  // whether the string is complete

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
  if(bleMouse.isConnected() && stringComplete) {
    //Serial.println(inputString);
    if(inputString == "ml") { // ML mouse left
      // mouse 0 (left click)
      bleMouse.click(MOUSE_LEFT);
    } else if(inputString == "mr") { // MR mouse right
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
      if(argc >= 2) {
        int x = args[0];
        int y = args[1];
        bleMouse.move(x,y);
        Serial.print("move x ");
        Serial.println(x);
        Serial.print("move y");
        Serial.println(y);
      }
    }
    
    inputString = "";
    argc = 0;
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
    static long receivedNumber = 0;
    static boolean negative = false;
    
    // get the new byte:
    char inChar = (char)Serial.read();
    // if the incoming character is a newline, set a flag so the main loop can
    // do something about it:

    switch(inChar)
    {
      case '\n':
        stringComplete = true;
        break;
      case endOfNumberDelimiter:
        if(negative)
          args[argc-1] = -receivedNumber;
        else
          args[argc-1] = receivedNumber;
        break;
      case startOfNumberDelimiter:
        argc++;
        receivedNumber = 0;
        negative = false;
        break;
      case '0' ... '9':
        receivedNumber *=10;
        receivedNumber += inChar - '0';
        break;
      case '-':
        negative = true;
        break;
      default:
        inputString += inChar;
    }
  }
}
