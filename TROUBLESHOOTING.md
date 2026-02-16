# Troubleshooting / Rozwiązywanie problemów

## Windows: "Windows protected your PC" / "System Windows ochronił ten komputer"

Because the installer is not signed with a paid certificate, Windows Defender SmartScreen may block it.
Ponieważ instalator nie jest podpisany płatnym certyfikatem, filtr Microsoft Defender SmartScreen może go zablokować.

**Solution / Rozwiązanie:**
1. Click **"More info"** / Kliknij **"Więcej informacji"**.
2. Click **"Run anyway"** / Kliknij **"Uruchom mimo to"**.

---

## macOS: "App is damaged" or "Unverified Developer" / "Aplikacja jest uszkodzona" lub "Niezweryfikowany deweloper"

Apple Silicon Macs require apps to be notarized. For this open-source tool, you can bypass this.
Komputery Mac z procesorami Apple wymagają notarialnego potwierdzenia aplikacji. W przypadku tego narzędzia open-source możesz to pominąć.

**Solution / Rozwiązanie:**

### 1. Right-click Open / Otwieranie przez prawy przycisk myszy
1. Locate the app in Finder / Znajdź aplikację w Finderze.
2. **Right-click** (Control-click) the app and select **Open** / Kliknij **prawym przyciskiem myszy** (lub z klawiszem Control) i wybierz **Otwórz**.
3. In the dialog, click **Open** / W oknie dialogowym kliknij **Otwórz**.

### 2. Move to Applications / Przenieś do Aplikacji
Ensure the app is in your `/Applications` folder before running.
Upewnij się, że aplikacja znajduje się w folderze `/Applications` przed uruchomieniem.

### 3. Clear Quarantine (if "Damaged") / Usunięcie kwarantanny (jeśli "Uszkodzona")
If macOS says the app is damaged, run this in Terminal:
Jeśli macOS twierdzi, że aplikacja jest uszkodzona, uruchom to w Terminalu:

```bash
xattr -d com.apple.quarantine /Applications/Walksnail\ OSD\ Tool.app
```

---

## Logs / Logi
If the app still doesn't open, please check the log file:
Jeśli aplikacja nadal się nie otwiera, sprawdź plik logów:

- **Windows**: `%APPDATA%\rs\walksnail-osd-tool\walksnail-osd-tool.log`
- **macOS**: `~/Library/Application Support/rs/walksnail-osd-tool/walksnail-osd-tool.log`
