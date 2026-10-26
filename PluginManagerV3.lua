local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1, L28_1, L29_1
L0_1 = "Hi friends. Nate here. This is just a quick note to ask that you please respect the time and effort I put into creating this software. Negative Lab Pro is not made or sold by a giant corporation. It is just me. It is the way that I personally provide for my family. Decompiling or modifying this software not only violates copyright law, but it directly hurts my ablity to keep making and improving this software for the film community. I understand that not everyone is in a position to purchase a Negative Lab Pro license. I do offer a 50% discount for students. And if your financial situation prohibits you from using Negative Lab Pro, please email me at nate@natephotographic.com. But please, leave this software as is so that I can continue making software for the film community. Thanks! -Nate"
L1_1 = import
L2_1 = "LrStringUtils"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrPathUtils"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrView"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrHttp"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrBinding"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrApplication"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrFileUtils"
L7_1 = L7_1(L8_1)
L8_1 = import
L9_1 = "LrColor"
L8_1 = L8_1(L9_1)
L9_1 = import
L10_1 = "LrLogger"
L9_1 = L9_1(L10_1)
L10_1 = import
L11_1 = "LrTasks"
L10_1 = L10_1(L11_1)
L11_1 = require
L12_1 = "UtilV3"
L11_1 = L11_1(L12_1)
L12_1 = import
L13_1 = "LrFunctionContext"
L12_1 = L12_1(L13_1)
L13_1 = require
L14_1 = "JSON"
L13_1 = L13_1(L14_1)
L14_1 = require
L15_1 = "NLPSessionV3"
L14_1 = L14_1(L15_1)
L15_1 = require
L16_1 = "inspect"
L15_1 = L15_1(L16_1)
L16_1 = import
L17_1 = "LrPrefs"
L16_1 = L16_1(L17_1)
L16_1 = L16_1.prefsForPlugin
L16_1 = L16_1()
L17_1 = L9_1
L18_1 = "Negative Lab"
L17_1 = L17_1(L18_1)
L18_1 = _G
L18_1 = L18_1.dev
if L18_1 == true then
  L19_1 = L17_1
  L18_1 = L17_1.enable
  L20_1 = "logfile"
  L18_1(L19_1, L20_1)
else
  L19_1 = L17_1
  L18_1 = L17_1.disable
  L18_1(L19_1)
end
L18_1 = L2_1.getStandardFilePath
L19_1 = "appData"
L18_1 = L18_1(L19_1)
L19_1 = L2_1.parent
L20_1 = L18_1
L19_1 = L19_1(L20_1)
L18_1 = L19_1
L19_1 = L2_1.parent
L20_1 = L18_1
L19_1 = L19_1(L20_1)
L18_1 = L19_1
L19_1 = L2_1.child
L20_1 = L18_1
L21_1 = "Negative Lab Pro"
L19_1 = L19_1(L20_1, L21_1)
L20_1 = L2_1.child
L21_1 = L19_1
L22_1 = "Backup"
L20_1 = L20_1(L21_1, L22_1)
L21_1 = "backup-license.txt"
L22_1 = MAC_ENV
if L22_1 then
  L22_1 = "/"
  if L22_1 then
    goto lbl_92
  end
end
L22_1 = "\\"
::lbl_92::
L23_1 = L20_1
L24_1 = L22_1
L25_1 = L21_1
L23_1 = L23_1 .. L24_1 .. L25_1
L24_1 = L11_1.v3
L24_1 = L24_1()
L25_1 = L24_1
L26_1 = "-k"
L25_1 = L25_1 .. L26_1
L26_1 = L24_1
L27_1 = "-i"
L26_1 = L26_1 .. L27_1

function L27_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2
  L1_2 = ""
  A0_2.enableActivation = false
  A0_2.activationError = "Submitting... hold on..."
  L2_2 = A0_2.serial
  if L2_2 ~= nil then
    L2_2 = A0_2.serial
    if L2_2 ~= "" then
      goto lbl_14
    end
  end
  A0_2.activationError = "ERROR: You must enter your serial number"
  A0_2.enableActivation = false
  do return end
  goto lbl_33
  ::lbl_14::
  L2_2 = 24
  L3_2 = L12_1
  L3_2 = L3_2.postAsyncTaskWithContext
  L4_2 = "posting credentials"
  
  function L5_2()
    local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3
    L0_3 = A0_2
    L0_3.activationError = "AUTHORIZING... PLEASE WAIT..."
    L0_3 = L11_1
    L0_3 = L0_3.activate
    L1_3 = A0_2
    L1_3 = L1_3.serial
    L0_3, L1_3 = L0_3(L1_3)
    L2_3 = L10_1
    L2_3 = L2_3.sleep
    L3_3 = 0
    L2_3(L3_3)
    if L0_3 == nil then
      L2_3 = A0_2
      L2_3.activationError = "ERROR: Could not connect. Check internet connection and turn off firewalls. "
      L2_3 = A0_2
      L2_3.enableActivation = true
      return
    end
    L2_3 = L13_1
    L3_3 = L2_3
    L2_3 = L2_3.decode
    L4_3 = L0_3
    L2_3 = L2_3(L3_3, L4_3)
    L3_3 = L17_1
    L4_3 = L3_3
    L3_3 = L3_3.trace
    L5_3 = "Response Table: "
    L6_3 = L15_1
    L7_3 = L2_3
    L6_3 = L6_3(L7_3)
    L5_3 = L5_3 .. L6_3
    L3_3(L4_3, L5_3)
    L3_3 = L11_1
    L3_3 = L3_3.keyIsInTable
    L4_3 = L2_3
    L5_3 = "activated"
    L3_3 = L3_3(L4_3, L5_3)
    if L3_3 then
      L3_3 = L11_1
      L3_3 = L3_3.keyIsInTable
      L4_3 = L2_3
      L5_3 = "meta"
      L3_3 = L3_3(L4_3, L5_3)
      if L3_3 then
        L3_3 = L17_1
        L4_3 = L3_3
        L3_3 = L3_3.trace
        L5_3 = "key found"
        L3_3(L4_3, L5_3)
    end
    else
      L3_3 = L17_1
      L4_3 = L3_3
      L3_3 = L3_3.trace
      L5_3 = "key not found"
      L3_3(L4_3, L5_3)
    end
    L3_3 = L11_1
    L3_3 = L3_3.keyIsInTable
    L4_3 = L2_3
    L5_3 = "activated"
    L3_3 = L3_3(L4_3, L5_3)
    if L3_3 then
      L3_3 = L2_3.activated
      if L3_3 == true then
        L3_3 = L11_1
        L3_3 = L3_3.keyIsInTable
        L4_3 = L2_3
        L5_3 = "meta"
        L3_3 = L3_3(L4_3, L5_3)
        if L3_3 then
          L3_3 = L2_3.meta
          L3_3 = L3_3.product_id
          if L3_3 ~= 55701 then
            L3_3 = L2_3.meta
            L3_3 = L3_3.product_id
            if L3_3 ~= 64248 then
              goto lbl_146
            end
          end
          L3_3 = L2_3.license_key
          L3_3 = L3_3.key
          L4_3 = L2_3.instance
          L4_3 = L4_3.id
          L5_3 = L2_3.meta
          L5_3 = L5_3.customer_email
          L6_3 = A0_2
          L6_3.isActivated = true
          L6_3 = A0_2
          L7_3 = L2_3.license_key
          L7_3 = L7_3.key
          L6_3.licenseKey = L7_3
          L6_3 = A0_2
          L7_3 = L2_3.meta
          L7_3 = L7_3.customer_email
          L6_3.email = L7_3
          L6_3 = A0_2
          L6_3.activationError = [[

 
 
 
 
 ]]
          L6_3 = A0_2
          L6_3.enableActivation = true
          L6_3 = L16_1
          L7_3 = L25_1
          L6_3[L7_3] = L3_3
          L6_3 = L16_1
          L7_3 = L26_1
          L6_3[L7_3] = L4_3
          L6_3 = L16_1
          L6_3.email = L5_3
          L6_3 = L16_1
          L16_1 = L6_3
          L6_3 = L10_1
          L6_3 = L6_3.sleep
          L7_3 = 0.5
          L6_3(L7_3)
          L6_3 = L5_3
          L7_3 = "\n"
          L8_3 = L3_3
          L9_3 = "\n"
          L10_3 = L4_3
          L11_3 = "\n"
          L12_3 = L11_1
          L12_3 = L12_3.v3
          L12_3 = L12_3()
          L6_3 = L6_3 .. L7_3 .. L8_3 .. L9_3 .. L10_3 .. L11_3 .. L12_3
          L1_2 = L6_3
          L6_3 = L7_1
          L6_3 = L6_3.exists
          L7_3 = L20_1
          L6_3 = L6_3(L7_3)
          if not L6_3 then
            L6_3 = L7_1
            L6_3 = L6_3.createAllDirectories
            L7_3 = L20_1
            L6_3(L7_3)
          end
          L6_3 = io
          L6_3 = L6_3.open
          L7_3 = L23_1
          L8_3 = "w"
          L6_3 = L6_3(L7_3, L8_3)
          L8_3 = L6_3
          L7_3 = L6_3.write
          L9_3 = L1_2
          L7_3(L8_3, L9_3)
          L8_3 = L6_3
          L7_3 = L6_3.close
          L7_3(L8_3)
          return
      end
    end
    else
      ::lbl_146::
      L3_3 = L17_1
      L4_3 = L3_3
      L3_3 = L3_3.trace
      L5_3 = "NOT TRUE"
      L3_3(L4_3, L5_3)
      L3_3 = L17_1
      L4_3 = L3_3
      L3_3 = L3_3.trace
      L5_3 = L15_1
      L6_3 = L2_3
      L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3 = L5_3(L6_3)
      L3_3(L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3)
      L3_3 = L11_1
      L3_3 = L3_3.keyIsInTable
      L4_3 = L2_3
      L5_3 = "license_key"
      L3_3 = L3_3(L4_3, L5_3)
      if L3_3 then
        L3_3 = L17_1
        L4_3 = L3_3
        L3_3 = L3_3.trace
        L5_3 = "license_key"
        L3_3(L4_3, L5_3)
        L3_3 = L2_3.license_key
        L3_3 = L3_3.activation_limit
        L4_3 = L2_3.license_key
        L4_3 = L4_3.activation_usage
        if L3_3 <= L4_3 then
          L5_3 = A0_2
          L5_3.activationError = [[
ERROR: Out of License Uses.
Visit https://app.lemonsqueezy.com/my-orders to remotely deactivate prior uses.]]
          L5_3 = A0_2
          L5_3.enableActivation = true
          return
        end
      else
        L3_3 = L11_1
        L3_3 = L3_3.keyIsInTable
        L4_3 = L2_3
        L5_3 = "error"
        L3_3 = L3_3(L4_3, L5_3)
        if L3_3 then
          L3_3 = L17_1
          L4_3 = L3_3
          L3_3 = L3_3.trace
          L5_3 = "error"
          L3_3(L4_3, L5_3)
          L3_3 = A0_2
          L4_3 = L2_3.error
          L3_3.activationError = L4_3
          L3_3 = A0_2
          L3_3.enableActivation = true
          return
        end
      end
    end
  end
  
  L3_2(L4_2, L5_2)
  ::lbl_33::
end

function L28_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = false
  L2_2 = L12_1
  L2_2 = L2_2.postAsyncTaskWithContext
  L3_2 = "posting credentials"
  
  function L4_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3
    L1_3 = A0_2
    L1_3.activationError = "Deactivating..."
    L2_3 = A0_3
    L1_3 = A0_3.addCleanupHandler
    
    function L3_3()
      local L0_4, L1_4
      L0_4 = L1_2
      if L0_4 == true then
        L0_4 = L16_1
        L1_4 = L25_1
        L0_4[L1_4] = nil
        L0_4 = L16_1
        L1_4 = L26_1
        L0_4[L1_4] = nil
        L0_4 = L16_1
        L0_4.email = nil
        L0_4 = L16_1
        L16_1 = L0_4
        L0_4 = L10_1
        L0_4 = L0_4.sleep
        L0_4()
        L0_4 = L16_1
        L16_1 = L0_4
      end
    end
    
    L1_3(L2_3, L3_3)
    L1_3 = nil
    L2_3 = nil
    L3_3 = L16_1
    L4_3 = L25_1
    L3_3 = L3_3[L4_3]
    if L3_3 ~= nil then
      L3_3 = L16_1
      L4_3 = L25_1
      L1_3 = L3_3[L4_3]
    end
    L3_3 = L16_1
    L4_3 = L26_1
    L3_3 = L3_3[L4_3]
    if L3_3 ~= nil then
      L3_3 = L16_1
      L4_3 = L26_1
      L2_3 = L3_3[L4_3]
    end
    if L1_3 == nil or L2_3 == nil then
      L3_3 = L14_1
      L3_3 = L3_3.getBackupLicense
      L3_3 = L3_3()
      L4_3 = nil
      if L3_3 ~= nil then
        L5_3 = L3_3.hash_table
        if L5_3 ~= nil then
          L5_3 = L3_3.hash_table
          L6_3 = "-k"
          L4_3 = L5_3 .. L6_3
          L5_3 = L3_3.hash_table
          L6_3 = "-i"
          L5_3 = L5_3 .. L6_3
          backup_instance_hash = L5_3
        end
      end
      if L4_3 ~= nil then
        L5_3 = L16_1
        L5_3 = L5_3[L4_3]
        if L5_3 ~= nil then
          L5_3 = L16_1
          L6_3 = backup_instance_hash
          L5_3 = L5_3[L6_3]
          if L5_3 ~= nil then
            L5_3 = L16_1
            L1_3 = L5_3[L4_3]
            L5_3 = L16_1
            L6_3 = backup_instance_hash
            L2_3 = L5_3[L6_3]
          end
        end
      end
    end
    if L1_3 ~= nil and L2_3 ~= nil then
      L3_3 = L11_1
      L3_3 = L3_3.deactivate
      L4_3 = L1_3
      L5_3 = L2_3
      L3_3, L4_3 = L3_3(L4_3, L5_3)
      if L3_3 == nil then
        L5_3 = A0_2
        L5_3.activationError = "ERROR: Could not connect. Check internet connection and turn off firewalls. "
        return
      end
      L5_3 = L13_1
      L6_3 = L5_3
      L5_3 = L5_3.decode
      L7_3 = L3_3
      L5_3 = L5_3(L6_3, L7_3)
      L6_3 = L5_3.deactivated
      if L6_3 == true then
        L6_3 = true
        L1_2 = L6_3
        L6_3 = A0_2
        L6_3.activationError = "Your license key was successfully removed"
        L6_3 = A0_2
        L6_3.isActivated = false
        L6_3 = L7_1
        L6_3 = L6_3.exists
        L7_3 = L23_1
        L6_3 = L6_3(L7_3)
        if L6_3 then
          L6_3 = L7_1
          L6_3 = L6_3.delete
          L7_3 = L23_1
          L6_3, L7_3 = L6_3(L7_3)
          if not L6_3 then
          end
        end
      else
        L6_3 = A0_2
        L7_3 = "ERROR: "
        L8_3 = L5_3.error
        L7_3 = L7_3 .. L8_3
        L6_3.activationError = L7_3
      end
    end
  end
  
  L2_2(L3_2, L4_2)
end

deactivate = L28_1
L28_1 = {}
PluginManager = L28_1
L28_1 = PluginManager

function L29_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2
  L2_2 = L14_1
  L2_2 = L2_2.setSessionStatus
  L2_2()
  L2_2 = _G
  L2_2 = L2_2.NLPSessionStatus
  L2_2 = L2_2 == 3
  A1_2.isActivated = L2_2
  A1_2.activationError = [[
 
 
 ]]
  L2_2 = L16_1
  L3_2 = L25_1
  L2_2 = L2_2[L3_2]
  if L2_2 ~= nil then
    L2_2 = L16_1
    L3_2 = L25_1
    L2_2 = L2_2[L3_2]
    A1_2.licenseKey = L2_2
  else
    L2_2 = L14_1
    L2_2 = L2_2.getBackupLicense
    L2_2 = L2_2()
    L3_2 = nil
    if L2_2 ~= nil then
      L4_2 = L2_2.hash_table
      if L4_2 ~= nil then
        L4_2 = L2_2.hash_table
        L5_2 = "-k"
        L3_2 = L4_2 .. L5_2
      end
    end
    if L3_2 ~= nil then
      L4_2 = L16_1
      L4_2 = L4_2[L3_2]
      if L4_2 ~= nil then
        L4_2 = L16_1
        L4_2 = L4_2[L3_2]
        A1_2.licenseKey = L4_2
    end
    else
      A1_2.licenseKey = ""
    end
  end
  L2_2 = L16_1
  L2_2 = L2_2.conversionLoggingEnabled
  if not L2_2 then
    L2_2 = false
  end
  A1_2.conversionLoggingEnabled = L2_2
  L2_2 = L16_1
  L2_2 = L2_2.email
  if not L2_2 then
    L2_2 = nil
  end
  A1_2.email = L2_2
  A1_2.enableActivation = false
  L3_2 = A1_2
  L2_2 = A1_2.addObserver
  L4_2 = "conversionLoggingEnabled"
  
  function L5_2(A0_3, A1_3, A2_3)
    local L3_3
    L3_3 = L16_1
    L3_3.conversionLoggingEnabled = A2_3
    L3_3 = L16_1
    L16_1 = L3_3
  end
  
  L2_2(L3_2, L4_2, L5_2)
  L3_2 = A0_2
  L2_2 = A0_2.column
  L4_2 = {}
  L4_2.bind_to_object = A1_2
  L6_2 = A0_2
  L5_2 = A0_2.static_text
  L7_2 = {}
  L7_2.font = "<system/bold>"
  L7_2.title = "Getting Started"
  L5_2 = L5_2(L6_2, L7_2)
  L7_2 = A0_2
  L6_2 = A0_2.spacer
  L8_2 = {}
  L8_2.height = 10
  L6_2 = L6_2(L7_2, L8_2)
  L8_2 = A0_2
  L7_2 = A0_2.static_text
  L9_2 = {}
  L9_2.title = "Learn more about film scanning best practices and how to use Negative Lab Pro in the official guide"
  L9_2.height_in_lines = 3
  L9_2.width_in_chars = 50
  L7_2 = L7_2(L8_2, L9_2)
  L9_2 = A0_2
  L8_2 = A0_2.push_button
  L10_2 = {}
  L10_2.font = "<system/small/bold>"
  L10_2.title = "OPEN GUIDE to Negative Lab Pro"
  
  function L11_2()
    local L0_3, L1_3
    L0_3 = L4_1
    L0_3 = L0_3.openUrlInBrowser
    L1_3 = "https://www.negativelabpro.com/guide"
    L0_3(L1_3)
  end
  
  L10_2.action = L11_2
  L11_2 = L8_1
  L12_2 = 0
  L13_2 = 0
  L14_2 = 1
  L11_2 = L11_2(L12_2, L13_2, L14_2)
  L10_2.text_color = L11_2
  L8_2 = L8_2(L9_2, L10_2)
  L10_2 = A0_2
  L9_2 = A0_2.spacer
  L11_2 = {}
  L11_2.height = 10
  L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L9_2(L10_2, L11_2)
  L4_2[1] = L5_2
  L4_2[2] = L6_2
  L4_2[3] = L7_2
  L4_2[4] = L8_2
  L4_2[5] = L9_2
  L4_2[6] = L10_2
  L4_2[7] = L11_2
  L4_2[8] = L12_2
  L4_2[9] = L13_2
  L4_2[10] = L14_2
  L4_2[11] = L15_2
  L4_2[12] = L16_2
  L4_2[13] = L17_2
  L4_2[14] = L18_2
  L4_2[15] = L19_2
  L4_2[16] = L20_2
  L4_2[17] = L21_2
  L4_2[18] = L22_2
  L4_2[19] = L23_2
  L4_2[20] = L24_2
  L4_2[21] = L25_2
  L4_2[22] = L26_2
  L4_2[23] = L27_2
  L4_2[24] = L28_2
  L4_2[25] = L29_2
  L2_2 = L2_2(L3_2, L4_2)
  L4_2 = A0_2
  L3_2 = A0_2.column
  L5_2 = {}
  L5_2.bind_to_object = A1_2
  L5_2.place = "overlapping"
  L5_2.margin = 3
  L7_2 = A0_2
  L6_2 = A0_2.view
  L8_2 = {}
  L9_2 = L5_1
  L9_2 = L9_2.keyEquals
  L10_2 = "isActivated"
  L11_2 = true
  L9_2 = L9_2(L10_2, L11_2)
  L8_2.visible = L9_2
  L10_2 = A0_2
  L9_2 = A0_2.row
  L11_2 = {}
  L11_2.margin_bottom = 15
  L13_2 = A0_2
  L12_2 = A0_2.static_text
  L14_2 = {}
  L14_2.font = "<system/bold>"
  L14_2.title = "You're AMAZING! Thanks for supporting Negative Lab Pro!"
  L14_2.alignment = "left"
  L14_2.fill_horizontal = 1
  L15_2 = L8_1
  L16_2 = 0
  L17_2 = 0.5
  L18_2 = 0
  L15_2 = L15_2(L16_2, L17_2, L18_2)
  L14_2.text_color = L15_2
  L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L12_2(L13_2, L14_2)
  L11_2[1] = L12_2
  L11_2[2] = L13_2
  L11_2[3] = L14_2
  L11_2[4] = L15_2
  L11_2[5] = L16_2
  L11_2[6] = L17_2
  L11_2[7] = L18_2
  L11_2[8] = L19_2
  L11_2[9] = L20_2
  L11_2[10] = L21_2
  L11_2[11] = L22_2
  L11_2[12] = L23_2
  L11_2[13] = L24_2
  L11_2[14] = L25_2
  L11_2[15] = L26_2
  L11_2[16] = L27_2
  L11_2[17] = L28_2
  L11_2[18] = L29_2
  L9_2 = L9_2(L10_2, L11_2)
  L11_2 = A0_2
  L10_2 = A0_2.row
  L12_2 = {}
  L14_2 = A0_2
  L13_2 = A0_2.label_spacing
  L13_2 = L13_2(L14_2)
  L12_2.spacing = L13_2
  L14_2 = A0_2
  L13_2 = A0_2.static_text
  L15_2 = {}
  L15_2.title = "V3 License Key: "
  L15_2.alignment = "right"
  L16_2 = L3_1
  L16_2 = L16_2.share
  L17_2 = "label_width"
  L16_2 = L16_2(L17_2)
  L15_2.width = L16_2
  L13_2 = L13_2(L14_2, L15_2)
  L15_2 = A0_2
  L14_2 = A0_2.static_text
  L16_2 = {}
  L17_2 = L3_1
  L17_2 = L17_2.bind
  L18_2 = "licenseKey"
  L17_2 = L17_2(L18_2)
  L16_2.title = L17_2
  L16_2.width_in_chars = 100
  L16_2.selectable = true
  L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L14_2(L15_2, L16_2)
  L12_2[1] = L13_2
  L12_2[2] = L14_2
  L12_2[3] = L15_2
  L12_2[4] = L16_2
  L12_2[5] = L17_2
  L12_2[6] = L18_2
  L12_2[7] = L19_2
  L12_2[8] = L20_2
  L12_2[9] = L21_2
  L12_2[10] = L22_2
  L12_2[11] = L23_2
  L12_2[12] = L24_2
  L12_2[13] = L25_2
  L12_2[14] = L26_2
  L12_2[15] = L27_2
  L12_2[16] = L28_2
  L12_2[17] = L29_2
  L10_2 = L10_2(L11_2, L12_2)
  L12_2 = A0_2
  L11_2 = A0_2.row
  L13_2 = {}
  L15_2 = A0_2
  L14_2 = A0_2.label_spacing
  L14_2 = L14_2(L15_2)
  L13_2.spacing = L14_2
  L15_2 = A0_2
  L14_2 = A0_2.static_text
  L16_2 = {}
  L16_2.title = "Registered to: "
  L16_2.alignment = "right"
  L17_2 = L3_1
  L17_2 = L17_2.share
  L18_2 = "label_width"
  L17_2 = L17_2(L18_2)
  L16_2.width = L17_2
  L14_2 = L14_2(L15_2, L16_2)
  L16_2 = A0_2
  L15_2 = A0_2.static_text
  L17_2 = {}
  L18_2 = L3_1
  L18_2 = L18_2.bind
  L19_2 = "email"
  L18_2 = L18_2(L19_2)
  L17_2.title = L18_2
  L17_2.width_in_chars = 100
  L17_2.selectable = true
  L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L15_2(L16_2, L17_2)
  L13_2[1] = L14_2
  L13_2[2] = L15_2
  L13_2[3] = L16_2
  L13_2[4] = L17_2
  L13_2[5] = L18_2
  L13_2[6] = L19_2
  L13_2[7] = L20_2
  L13_2[8] = L21_2
  L13_2[9] = L22_2
  L13_2[10] = L23_2
  L13_2[11] = L24_2
  L13_2[12] = L25_2
  L13_2[13] = L26_2
  L13_2[14] = L27_2
  L13_2[15] = L28_2
  L13_2[16] = L29_2
  L11_2 = L11_2(L12_2, L13_2)
  L13_2 = A0_2
  L12_2 = A0_2.spacer
  L14_2 = {}
  L14_2.height = 10
  L12_2 = L12_2(L13_2, L14_2)
  L14_2 = A0_2
  L13_2 = A0_2.row
  L15_2 = {}
  L17_2 = A0_2
  L16_2 = A0_2.push_button
  L18_2 = {}
  L18_2.width = 150
  L18_2.title = "Deactivate This Device"
  L18_2.enabled = true
  
  function L19_2()
    local L0_3, L1_3
    L0_3 = deactivate
    L1_3 = A1_2
    L0_3(L1_3)
  end
  
  L18_2.action = L19_2
  L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L16_2(L17_2, L18_2)
  L15_2[1] = L16_2
  L15_2[2] = L17_2
  L15_2[3] = L18_2
  L15_2[4] = L19_2
  L15_2[5] = L20_2
  L15_2[6] = L21_2
  L15_2[7] = L22_2
  L15_2[8] = L23_2
  L15_2[9] = L24_2
  L15_2[10] = L25_2
  L15_2[11] = L26_2
  L15_2[12] = L27_2
  L15_2[13] = L28_2
  L15_2[14] = L29_2
  L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L13_2(L14_2, L15_2)
  L8_2[1] = L9_2
  L8_2[2] = L10_2
  L8_2[3] = L11_2
  L8_2[4] = L12_2
  L8_2[5] = L13_2
  L8_2[6] = L14_2
  L8_2[7] = L15_2
  L8_2[8] = L16_2
  L8_2[9] = L17_2
  L8_2[10] = L18_2
  L8_2[11] = L19_2
  L8_2[12] = L20_2
  L8_2[13] = L21_2
  L8_2[14] = L22_2
  L8_2[15] = L23_2
  L8_2[16] = L24_2
  L8_2[17] = L25_2
  L8_2[18] = L26_2
  L8_2[19] = L27_2
  L8_2[20] = L28_2
  L8_2[21] = L29_2
  L6_2 = L6_2(L7_2, L8_2)
  L8_2 = A0_2
  L7_2 = A0_2.view
  L9_2 = {}
  L10_2 = L5_1
  L10_2 = L10_2.keyIsNot
  L11_2 = "isActivated"
  L12_2 = true
  L10_2 = L10_2(L11_2, L12_2)
  L9_2.visible = L10_2
  L11_2 = A0_2
  L10_2 = A0_2.static_text
  L12_2 = {}
  L12_2.font = "<system/bold>"
  L12_2.title = "Unlock the Full Version"
  L12_2.alignment = "left"
  L10_2 = L10_2(L11_2, L12_2)
  L12_2 = A0_2
  L11_2 = A0_2.spacer
  L13_2 = {}
  L13_2.height = 10
  L11_2 = L11_2(L12_2, L13_2)
  L13_2 = A0_2
  L12_2 = A0_2.static_text
  L14_2 = {}
  L14_2.title = [[
You are currently in TRIAL MODE, which allows you to preview your edits
and apply on to up to 12 negatives. To unlock, you must purchase a license key.]]
  L14_2.height_in_lines = 3
  L14_2.width_in_chars = 50
  L12_2 = L12_2(L13_2, L14_2)
  L14_2 = A0_2
  L13_2 = A0_2.push_button
  L15_2 = {}
  L15_2.title = "Purchase a License"
  L15_2.font = "<system/small/bold>"
  
  function L16_2()
    local L0_3, L1_3
    L0_3 = L4_1
    L0_3 = L0_3.openUrlInBrowser
    L1_3 = "https://www.negativelabpro.com/pricing"
    L0_3(L1_3)
  end
  
  L15_2.action = L16_2
  L16_2 = L8_1
  L17_2 = 0
  L18_2 = 0
  L19_2 = 1
  L16_2 = L16_2(L17_2, L18_2, L19_2)
  L15_2.text_color = L16_2
  L13_2 = L13_2(L14_2, L15_2)
  L15_2 = A0_2
  L14_2 = A0_2.spacer
  L16_2 = {}
  L16_2.height = 20
  L14_2 = L14_2(L15_2, L16_2)
  L16_2 = A0_2
  L15_2 = A0_2.static_text
  L17_2 = {}
  L17_2.font = "<system/bold>"
  L17_2.title = "Activate License Key:"
  L17_2.alignment = "left"
  L15_2 = L15_2(L16_2, L17_2)
  L17_2 = A0_2
  L16_2 = A0_2.static_text
  L18_2 = {}
  L18_2.wraps = true
  L18_2.width_in_chars = 50
  L18_2.height_in_lines = 4
  L18_2.title = "After purchase, your license can be found in your email with the subject line 'You Got Negative Lab Pro!' Copy and paste the license key directly from the email, then hit ACTIVATE LICENCE (the button will only become enabled once you've entered the full license.)"
  L19_2 = L8_1
  L20_2 = 0.5
  L21_2 = 0.5
  L22_2 = 0.5
  L19_2 = L19_2(L20_2, L21_2, L22_2)
  L18_2.text_color = L19_2
  L16_2 = L16_2(L17_2, L18_2)
  L18_2 = A0_2
  L17_2 = A0_2.spacer
  L19_2 = {}
  L19_2.height = 5
  L17_2 = L17_2(L18_2, L19_2)
  L19_2 = A0_2
  L18_2 = A0_2.row
  L20_2 = {}
  L22_2 = A0_2
  L21_2 = A0_2.picture
  L23_2 = {}
  L24_2 = L2_1
  L24_2 = L24_2.child
  L25_2 = _PLUGIN
  L25_2 = L25_2.path
  L26_2 = "img/key.png"
  L24_2 = L24_2(L25_2, L26_2)
  L23_2.value = L24_2
  L23_2.height = 22
  L23_2.width = 22
  L21_2 = L21_2(L22_2, L23_2)
  L23_2 = A0_2
  L22_2 = A0_2.edit_field
  L24_2 = {}
  L24_2.font = "<system>"
  L24_2.width_in_chars = 30
  L24_2.height_in_lines = 3
  L25_2 = L3_1
  L25_2 = L25_2.bind
  L26_2 = "serial"
  L25_2 = L25_2(L26_2)
  L24_2.value = L25_2
  L24_2.immediate = true
  
  function L25_2(A0_3, A1_3)
    local L2_3, L3_3, L4_3
    L2_3 = L1_1
    L2_3 = L2_3.trimWhitespace
    L3_3 = A1_3
    L2_3 = L2_3(L3_3)
    A1_3 = L2_3
    L2_3 = string
    L2_3 = L2_3.match
    L3_3 = A1_3
    L4_3 = "%w%w%w%w%w%w%w%w%-%w%w%w%w%-%w%w%w%w%-%w%w%w%w%-%w%w%w%w%w%w%w%w%w%w%w%w"
    L2_3 = L2_3(L3_3, L4_3)
    if L2_3 then
      L2_3 = #A1_3
      if L2_3 == 36 then
        L2_3 = A1_2
        L2_3.enableActivation = true
        L2_3 = A1_2
        L2_3.activationError = " "
        L2_3 = true
        L3_3 = A1_3
        L4_3 = "This string matches!"
        return L2_3, L3_3, L4_3
    end
    else
      L2_3 = string
      L2_3 = L2_3.match
      L3_3 = A1_3
      L4_3 = "%w%w%w%w%w%w%w%w%-%w%w%w%w%w%w%w%w%-%w%w%w%w%w%w%w%w%-%w%w%w%w%w%w%w%w"
      L2_3 = L2_3(L3_3, L4_3)
      if L2_3 then
        L2_3 = #A1_3
        if L2_3 == 35 then
          L2_3 = A1_2
          L2_3.enableActivation = false
          L2_3 = A1_2
          L2_3.activationError = [[
ERROR: V2 LICENSE DETECTED
V2 license keys are not compatible with V3. You can download V2 at https://www.negativelabpro.com/download-for or upgrade to V3 at https://www.negativelabpro.com/pricing.]]
      end
      else
        L2_3 = A1_2
        L2_3.enableActivation = false
        L2_3 = A1_2
        L2_3.activationError = " "
        L2_3 = false
        L3_3 = A1_3
        return L2_3, L3_3
      end
    end
  end
  
  L24_2.validate = L25_2
  L22_2 = L22_2(L23_2, L24_2)
  L24_2 = A0_2
  L23_2 = A0_2.push_button
  L25_2 = {}
  L25_2.font = "<system/bold>"
  L25_2.title = "ACTIVATE"
  L26_2 = L5_1
  L26_2 = L26_2.keyEquals
  L27_2 = "enableActivation"
  L28_2 = true
  L26_2 = L26_2(L27_2, L28_2)
  L25_2.enabled = L26_2
  
  function L26_2()
    local L0_3, L1_3
    L0_3 = A1_2
    L0_3.enableActivation = false
    L0_3 = L27_1
    L1_3 = A1_2
    L0_3(L1_3)
  end
  
  L25_2.action = L26_2
  L26_2 = L8_1
  L27_2 = 0
  L28_2 = 0
  L29_2 = 1
  L26_2 = L26_2(L27_2, L28_2, L29_2)
  L25_2.text_color = L26_2
  L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L23_2(L24_2, L25_2)
  L20_2[1] = L21_2
  L20_2[2] = L22_2
  L20_2[3] = L23_2
  L20_2[4] = L24_2
  L20_2[5] = L25_2
  L20_2[6] = L26_2
  L20_2[7] = L27_2
  L20_2[8] = L28_2
  L20_2[9] = L29_2
  L18_2 = L18_2(L19_2, L20_2)
  L20_2 = A0_2
  L19_2 = A0_2.spacer
  L21_2 = {}
  L21_2.height = 8
  L19_2 = L19_2(L20_2, L21_2)
  L21_2 = A0_2
  L20_2 = A0_2.spacer
  L22_2 = {}
  L22_2.height = 10
  L20_2 = L20_2(L21_2, L22_2)
  L22_2 = A0_2
  L21_2 = A0_2.static_text
  L23_2 = {}
  L24_2 = L3_1
  L24_2 = L24_2.bind
  L25_2 = "activationError"
  L24_2 = L24_2(L25_2)
  L23_2.title = L24_2
  L23_2.font = "<system/small/bold>"
  L23_2.alignment = "left"
  L23_2.selectable = true
  L23_2.fill_horizontal = 1
  L24_2 = L8_1
  L25_2 = "red"
  L24_2 = L24_2(L25_2)
  L23_2.text_color = L24_2
  L23_2.height_in_lines = 4
  L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L21_2(L22_2, L23_2)
  L9_2[1] = L10_2
  L9_2[2] = L11_2
  L9_2[3] = L12_2
  L9_2[4] = L13_2
  L9_2[5] = L14_2
  L9_2[6] = L15_2
  L9_2[7] = L16_2
  L9_2[8] = L17_2
  L9_2[9] = L18_2
  L9_2[10] = L19_2
  L9_2[11] = L20_2
  L9_2[12] = L21_2
  L9_2[13] = L22_2
  L9_2[14] = L23_2
  L9_2[15] = L24_2
  L9_2[16] = L25_2
  L9_2[17] = L26_2
  L9_2[18] = L27_2
  L9_2[19] = L28_2
  L9_2[20] = L29_2
  L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L7_2(L8_2, L9_2)
  L5_2[1] = L6_2
  L5_2[2] = L7_2
  L5_2[3] = L8_2
  L5_2[4] = L9_2
  L5_2[5] = L10_2
  L5_2[6] = L11_2
  L5_2[7] = L12_2
  L5_2[8] = L13_2
  L5_2[9] = L14_2
  L5_2[10] = L15_2
  L5_2[11] = L16_2
  L5_2[12] = L17_2
  L5_2[13] = L18_2
  L5_2[14] = L19_2
  L5_2[15] = L20_2
  L5_2[16] = L21_2
  L5_2[17] = L22_2
  L5_2[18] = L23_2
  L5_2[19] = L24_2
  L5_2[20] = L25_2
  L5_2[21] = L26_2
  L5_2[22] = L27_2
  L5_2[23] = L28_2
  L5_2[24] = L29_2
  L3_2 = L3_2(L4_2, L5_2)
  L5_2 = A0_2
  L4_2 = A0_2.column
  L6_2 = {}
  L6_2.bind_to_object = A1_2
  L6_2.margin = 3
  L8_2 = A0_2
  L7_2 = A0_2.view
  L9_2 = {}
  L10_2 = L5_1
  L10_2 = L10_2.keyEquals
  L11_2 = "isActivated"
  L12_2 = true
  L10_2 = L10_2(L11_2, L12_2)
  L9_2.visible = L10_2
  L11_2 = A0_2
  L10_2 = A0_2.row
  L12_2 = {}
  L12_2.margin_bottom = 15
  L14_2 = A0_2
  L13_2 = A0_2.static_text
  L15_2 = {}
  L15_2.font = "<system/bold>"
  L15_2.title = "You're AMAZING! Thanks for supporting Negative Lab Pro!"
  L15_2.alignment = "left"
  L15_2.fill_horizontal = 1
  L16_2 = L8_1
  L17_2 = 0
  L18_2 = 0.5
  L19_2 = 0
  L16_2 = L16_2(L17_2, L18_2, L19_2)
  L15_2.text_color = L16_2
  L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L13_2(L14_2, L15_2)
  L12_2[1] = L13_2
  L12_2[2] = L14_2
  L12_2[3] = L15_2
  L12_2[4] = L16_2
  L12_2[5] = L17_2
  L12_2[6] = L18_2
  L12_2[7] = L19_2
  L12_2[8] = L20_2
  L12_2[9] = L21_2
  L12_2[10] = L22_2
  L12_2[11] = L23_2
  L12_2[12] = L24_2
  L12_2[13] = L25_2
  L12_2[14] = L26_2
  L12_2[15] = L27_2
  L12_2[16] = L28_2
  L12_2[17] = L29_2
  L10_2 = L10_2(L11_2, L12_2)
  L12_2 = A0_2
  L11_2 = A0_2.row
  L13_2 = {}
  L15_2 = A0_2
  L14_2 = A0_2.label_spacing
  L14_2 = L14_2(L15_2)
  L13_2.spacing = L14_2
  L15_2 = A0_2
  L14_2 = A0_2.static_text
  L16_2 = {}
  L16_2.title = "V3 License Key: "
  L16_2.alignment = "right"
  L17_2 = L3_1
  L17_2 = L17_2.share
  L18_2 = "label_width"
  L17_2 = L17_2(L18_2)
  L16_2.width = L17_2
  L14_2 = L14_2(L15_2, L16_2)
  L16_2 = A0_2
  L15_2 = A0_2.static_text
  L17_2 = {}
  L18_2 = L3_1
  L18_2 = L18_2.bind
  L19_2 = "licenseKey"
  L18_2 = L18_2(L19_2)
  L17_2.title = L18_2
  L17_2.width_in_chars = 100
  L17_2.selectable = true
  L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L15_2(L16_2, L17_2)
  L13_2[1] = L14_2
  L13_2[2] = L15_2
  L13_2[3] = L16_2
  L13_2[4] = L17_2
  L13_2[5] = L18_2
  L13_2[6] = L19_2
  L13_2[7] = L20_2
  L13_2[8] = L21_2
  L13_2[9] = L22_2
  L13_2[10] = L23_2
  L13_2[11] = L24_2
  L13_2[12] = L25_2
  L13_2[13] = L26_2
  L13_2[14] = L27_2
  L13_2[15] = L28_2
  L13_2[16] = L29_2
  L11_2 = L11_2(L12_2, L13_2)
  L13_2 = A0_2
  L12_2 = A0_2.row
  L14_2 = {}
  L16_2 = A0_2
  L15_2 = A0_2.label_spacing
  L15_2 = L15_2(L16_2)
  L14_2.spacing = L15_2
  L16_2 = A0_2
  L15_2 = A0_2.static_text
  L17_2 = {}
  L17_2.title = "Registered to: "
  L17_2.alignment = "right"
  L18_2 = L3_1
  L18_2 = L18_2.share
  L19_2 = "label_width"
  L18_2 = L18_2(L19_2)
  L17_2.width = L18_2
  L15_2 = L15_2(L16_2, L17_2)
  L17_2 = A0_2
  L16_2 = A0_2.static_text
  L18_2 = {}
  L19_2 = L3_1
  L19_2 = L19_2.bind
  L20_2 = "email"
  L19_2 = L19_2(L20_2)
  L18_2.title = L19_2
  L18_2.width_in_chars = 100
  L18_2.selectable = true
  L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L16_2(L17_2, L18_2)
  L14_2[1] = L15_2
  L14_2[2] = L16_2
  L14_2[3] = L17_2
  L14_2[4] = L18_2
  L14_2[5] = L19_2
  L14_2[6] = L20_2
  L14_2[7] = L21_2
  L14_2[8] = L22_2
  L14_2[9] = L23_2
  L14_2[10] = L24_2
  L14_2[11] = L25_2
  L14_2[12] = L26_2
  L14_2[13] = L27_2
  L14_2[14] = L28_2
  L14_2[15] = L29_2
  L12_2 = L12_2(L13_2, L14_2)
  L14_2 = A0_2
  L13_2 = A0_2.spacer
  L15_2 = {}
  L15_2.height = 10
  L13_2 = L13_2(L14_2, L15_2)
  L15_2 = A0_2
  L14_2 = A0_2.row
  L16_2 = {}
  L18_2 = A0_2
  L17_2 = A0_2.push_button
  L19_2 = {}
  L19_2.width = 150
  L19_2.title = "Deactivate This Device"
  L19_2.enabled = true
  
  function L20_2()
    local L0_3, L1_3
    L0_3 = deactivate
    L1_3 = A1_2
    L0_3(L1_3)
  end
  
  L19_2.action = L20_2
  L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L17_2(L18_2, L19_2)
  L16_2[1] = L17_2
  L16_2[2] = L18_2
  L16_2[3] = L19_2
  L16_2[4] = L20_2
  L16_2[5] = L21_2
  L16_2[6] = L22_2
  L16_2[7] = L23_2
  L16_2[8] = L24_2
  L16_2[9] = L25_2
  L16_2[10] = L26_2
  L16_2[11] = L27_2
  L16_2[12] = L28_2
  L16_2[13] = L29_2
  L14_2 = L14_2(L15_2, L16_2)
  L16_2 = A0_2
  L15_2 = A0_2.spacer
  L17_2 = {}
  L17_2.height = 10
  L15_2 = L15_2(L16_2, L17_2)
  L17_2 = A0_2
  L16_2 = A0_2.static_text
  L18_2 = {}
  L19_2 = L3_1
  L19_2 = L19_2.bind
  L20_2 = "activationError"
  L19_2 = L19_2(L20_2)
  L18_2.title = L19_2
  L18_2.selectable = true
  L18_2.alignment = "left"
  L18_2.fill_horizontal = 1
  L19_2 = L8_1
  L20_2 = "red"
  L19_2 = L19_2(L20_2)
  L18_2.text_color = L19_2
  L18_2.height_in_lines = 3
  L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L16_2(L17_2, L18_2)
  L9_2[1] = L10_2
  L9_2[2] = L11_2
  L9_2[3] = L12_2
  L9_2[4] = L13_2
  L9_2[5] = L14_2
  L9_2[6] = L15_2
  L9_2[7] = L16_2
  L9_2[8] = L17_2
  L9_2[9] = L18_2
  L9_2[10] = L19_2
  L9_2[11] = L20_2
  L9_2[12] = L21_2
  L9_2[13] = L22_2
  L9_2[14] = L23_2
  L9_2[15] = L24_2
  L9_2[16] = L25_2
  L9_2[17] = L26_2
  L9_2[18] = L27_2
  L9_2[19] = L28_2
  L9_2[20] = L29_2
  L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L7_2(L8_2, L9_2)
  L6_2[1] = L7_2
  L6_2[2] = L8_2
  L6_2[3] = L9_2
  L6_2[4] = L10_2
  L6_2[5] = L11_2
  L6_2[6] = L12_2
  L6_2[7] = L13_2
  L6_2[8] = L14_2
  L6_2[9] = L15_2
  L6_2[10] = L16_2
  L6_2[11] = L17_2
  L6_2[12] = L18_2
  L6_2[13] = L19_2
  L6_2[14] = L20_2
  L6_2[15] = L21_2
  L6_2[16] = L22_2
  L6_2[17] = L23_2
  L6_2[18] = L24_2
  L6_2[19] = L25_2
  L6_2[20] = L26_2
  L6_2[21] = L27_2
  L6_2[22] = L28_2
  L6_2[23] = L29_2
  L4_2 = L4_2(L5_2, L6_2)
  L6_2 = A0_2
  L5_2 = A0_2.column
  L7_2 = {}
  L7_2.bind_to_object = A1_2
  L9_2 = A0_2
  L8_2 = A0_2.checkbox
  L10_2 = {}
  L10_2.title = "Show Error Log During Conversion"
  L11_2 = L3_1
  L11_2 = L11_2.bind
  L12_2 = "conversionLoggingEnabled"
  L11_2 = L11_2(L12_2)
  L10_2.value = L11_2
  L10_2.tooltip = "Display more details when errors occur during batch conversions"
  L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2 = L8_2(L9_2, L10_2)
  L7_2[1] = L8_2
  L7_2[2] = L9_2
  L7_2[3] = L10_2
  L7_2[4] = L11_2
  L7_2[5] = L12_2
  L7_2[6] = L13_2
  L7_2[7] = L14_2
  L7_2[8] = L15_2
  L7_2[9] = L16_2
  L7_2[10] = L17_2
  L7_2[11] = L18_2
  L7_2[12] = L19_2
  L7_2[13] = L20_2
  L7_2[14] = L21_2
  L7_2[15] = L22_2
  L7_2[16] = L23_2
  L7_2[17] = L24_2
  L7_2[18] = L25_2
  L7_2[19] = L26_2
  L7_2[20] = L27_2
  L7_2[21] = L28_2
  L7_2[22] = L29_2
  L5_2 = L5_2(L6_2, L7_2)
  L6_2 = A1_2.isActivated
  if L6_2 == true then
    L6_2 = {}
    L7_2 = {}
    L7_2.title = "Negative Lab Pro v3"
    L8_2 = L2_2
    L9_2 = L4_2
    L7_2[1] = L8_2
    L7_2[2] = L9_2
    L8_2 = {}
    L8_2.title = "Options"
    L9_2 = L5_2
    L8_2[1] = L9_2
    L6_2[1] = L7_2
    L6_2[2] = L8_2
    return L6_2
  else
    L6_2 = {}
    L7_2 = {}
    L7_2.title = "Negative Lab Pro v3"
    L8_2 = L2_2
    L9_2 = L3_2
    L7_2[1] = L8_2
    L7_2[2] = L9_2
    L8_2 = {}
    L8_2.title = "Options"
    L9_2 = L5_2
    L8_2[1] = L9_2
    L6_2[1] = L7_2
    L6_2[2] = L8_2
    return L6_2
  end
end

L28_1.sectionsForTopOfDialog = L29_1
