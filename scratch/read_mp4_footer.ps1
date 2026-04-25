$path = 'g:\_temp\Artlynk\2026-04-25 Update 2.0.5\Video001.mp4'
$out = 'g:\Walksnail-OSD-Tool\scratch\end_of_mp4.txt'
$fs = New-Object System.IO.FileStream($path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read)
$fs.Seek(-50000, [System.IO.SeekOrigin]::End)
$buffer = New-Object byte[] 50000
$fs.Read($buffer, 0, 50000)
$fs.Close()
[System.Text.Encoding]::ASCII.GetString($buffer) | Out-File -FilePath $out
