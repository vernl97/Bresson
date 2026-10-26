local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1
L0_1 = import
L1_1 = "LrLogger"
L0_1 = L0_1(L1_1)
L1_1 = import
L2_1 = "LrHttp"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrFunctionContext"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrTasks"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrErrors"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrBinding"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrDialogs"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrView"
L7_1 = L7_1(L8_1)
L8_1 = require
L9_1 = "JSON"
L8_1 = L8_1(L9_1)
L9_1 = L0_1
L10_1 = "authenticator"
L9_1 = L9_1(L10_1)
L10_1 = _G
L10_1 = L10_1.dev
if L10_1 == true then
  L11_1 = L9_1
  L10_1 = L9_1.enable
  L12_1 = "logfile"
  L10_1(L11_1, L12_1)
else
  L11_1 = L9_1
  L10_1 = L9_1.disable
  L10_1(L11_1)
end
L10_1 = 12
L11_1 = import
L12_1 = "LrPrefs"
L11_1 = L11_1(L12_1)
L11_1 = L11_1.prefsForPlugin
L11_1 = L11_1()
L12_1 = {}
NATEAuthenticator = L12_1
L12_1 = NATEAuthenticator

function L13_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = L2_1
  L1_2 = L1_2.postAsyncTaskWithContext
  L2_2 = "posting credentials"
  
  function L3_2()
    local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3
    L0_3 = L11_1
    L0_3.activationError = "AUTHORIZING... PLEASE WAIT..."
    L0_3 = A0_2
    if L0_3 ~= nil then
      L0_3 = A0_2
      if L0_3 ~= "" then
        goto lbl_18
      end
    end
    L0_3 = L11_1
    L0_3.activationError = "ERROR: You must enter your serial number"
    L0_3 = L11_1
    L0_3.enableActivation = true
    L0_3 = L3_1
    L0_3 = L0_3.sleep
    L1_3 = 0.5
    L0_3(L1_3)
    do return end
    ::lbl_18::
    L0_3 = "https://api.gumroad.com/v2/licenses/verify"
    L1_3 = "product_permalink=negative-lab&license_key="
    L2_3 = A0_2
    L1_3 = L1_3 .. L2_3
    L2_3 = {}
    L3_3 = auth_header
    L4_3 = {}
    L4_3.field = "Content-Type"
    L4_3.value = "application/x-www-form-urlencoded"
    L5_3 = {}
    L5_3.field = "User-Agent"
    L5_3.value = "Lightroom Plugin"
    L6_3 = {}
    L6_3.field = "Cookie"
    L6_3.value = "GARBAGE"
    L2_3[1] = L3_3
    L2_3[2] = L4_3
    L2_3[3] = L5_3
    L2_3[4] = L6_3
    L3_3 = L1_1
    L3_3 = L3_3.post
    L4_3 = L0_3
    L5_3 = L1_3
    L6_3 = L2_3
    L7_3 = "POST"
    L8_3 = 10
    L3_3, L4_3 = L3_3(L4_3, L5_3, L6_3, L7_3, L8_3)
    if L3_3 == nil then
      L5_3 = L11_1
      L5_3.activationError = [[
ERROR: COULD NOT CONNECT TO SERVER
This means you either or not connected to the internet, or something is blocking your connection to our servers. Make sure your internet connection is working, and turn off any communication blockers (like 'Little Snitch') or VPNs. Then try again.]]
      L5_3 = L11_1
      L5_3.enableActivation = true
      L5_3 = L3_1
      L5_3 = L5_3.sleep
      L6_3 = 0.5
      L5_3(L6_3)
      return
    end
    L5_3 = L8_1
    L6_3 = L5_3
    L5_3 = L5_3.decode
    L7_3 = L3_3
    L5_3 = L5_3(L6_3, L7_3)
    L6_3 = L9_1
    L7_3 = L6_3
    L6_3 = L6_3.info
    L8_3 = L5_3.success
    L6_3(L7_3, L8_3)
    L6_3 = L5_3.success
    if L6_3 == true then
      L6_3 = L5_3.purchase
      L6_3 = L6_3.refunded
      if L6_3 == false then
        L6_3 = L5_3.uses
        L7_3 = L10_1
        if L6_3 < L7_3 then
          L6_3 = L11_1
          L6_3.isActivated = true
          L6_3 = L11_1
          L7_3 = A0_2
          L6_3.licenseKey = L7_3
          L6_3 = L11_1
          L6_3.activationError = ""
          L6_3 = L11_1
          L7_3 = L5_3.purchase
          L7_3 = L7_3.email
          L6_3.email = L7_3
          L6_3 = L11_1
          L6_3.enableActivation = true
          L6_3 = L3_1
          L6_3 = L6_3.sleep
          L7_3 = 0.5
          L6_3(L7_3)
          return
      end
    end
    else
      L6_3 = L5_3.success
      if L6_3 == false then
        L6_3 = L11_1
        L6_3.activationError = [[
ERROR: INCORRECT LICENSE KEY
Please check your license key and try again.]]
        L6_3 = L11_1
        L6_3.enableActivation = true
        L6_3 = L3_1
        L6_3 = L6_3.sleep
        L7_3 = 0.5
        L6_3(L7_3)
        return
      else
        L6_3 = L5_3.uses
        L7_3 = L10_1
        if L6_3 >= L7_3 then
          L6_3 = L11_1
          L6_3.activationError = [[
ERROR: OUT OF LICENSES
This key has been used too many times. Contact nate@natephotographic.com]]
          L6_3 = L11_1
          L6_3.enableActivation = true
          L6_3 = L3_1
          L6_3 = L6_3.sleep
          L7_3 = 0.5
          L6_3(L7_3)
          return
        end
      end
    end
  end
  
  L1_2(L2_2, L3_2)
end

L12_1.authenticate = L13_1
