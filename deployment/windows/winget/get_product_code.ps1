# From https://stackoverflow.com/a/31919297

$path = $args[0]

$comObjWI = New-Object -ComObject WindowsInstaller.Installer
$MSIDatabase = $comObjWI.GetType().InvokeMember("OpenDatabase","InvokeMethod",$Null,$comObjWI,@($Path,0))
$Query = "SELECT Value FROM Property WHERE Property = 'ProductCode'"
$View = $MSIDatabase.GetType().InvokeMember("OpenView","InvokeMethod",$null,$MSIDatabase,($Query))
$View.GetType().InvokeMember("Execute", "InvokeMethod", $null, $View, $null)
$Record = $View.GetType().InvokeMember("Fetch","InvokeMethod",$null,$View,$null)
$Value = $Record.GetType().InvokeMember("StringData","GetProperty",$null,$Record,1)

echo $Value