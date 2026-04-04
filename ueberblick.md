# Überblick

## Admin-Oberfläche
Die Verwaltung von Inhalten finden auf der Admin-Oberfläche statt.
Zuerst legst du Artikel an und 
weist diesen dann entsprechende Ausleihzeitfenster zu. 
Ausleihfenster definierst du durch das Anlegen einer Zeitspanne im Kalender 
und kannst du zusätzlich die Einschränkungen (Buchbar, Urlaub, Reparatur) festlegen. 
Durch das Kalenderfenster haben wir flexiblere Möglichkeiten die Ausleihe zu steuern und zu begrenzen.
Momentan ist die Adminseite durch eine Authentifizierung nicht geschützt. 
Ein Passwortschutz ist unbedingt erforderlich in der Produktion.

### Artikel anlegen 

Nutze das Formular auf der Artikelseite für die Detailbeschreibung deines Artikels (Ausstattung, Maße, Eigenschaften, Einsatzzwecke). Diese Beschreibung wird auf der Artikeldetailseite angezeigt. 
Ergänze unbedingt ein Bild des Artikels. 

### Buchungszeiträume verwalten 

Ein Artikel wird buchbar durch eine Verknüpfung von Artikel mithilfe eines Zeitrahmens.
Der Zeitrahmen definiert für den Buchungszeitraum ein Zeitfenster (Start/Ende).

So arbeitest du mit den Buchungszeiträumen:

1. Öffne **Neue Buchung** (eigene Seite).
2. Wähle **Artikel**.
3. Lege im Kalender eine **Zeitspanne** fest (Start/Ende sind Pflicht).
4. Wähle den **Typ**:
   - **Buchbar**: Zeitraum ist grundsätzlich nutzbar.
   - **Urlaub**: Zeitraum ist blockiert (z.B. Station geschlossen).
   - **Reparatur**: Zeitraum ist blockiert (z.B. Artikel defekt).
5. Speichere die Buchung. Anschließend kannst du sie in der Detailansicht bearbeiten.

### Buchungen auflisten

In der Listenansicht siehst du alle Buchungen paginiert. Du kannst außerdem über ein Suchfeld
nach **Artikelnamen** filtern.

- Klick auf eine Zeile öffnet die **Detailansicht** der Buchung (Bearbeiten).
- Über **Neue Buchung** legst du eine Buchung auf einer separaten Seite an.


## Nutzer Oberfläche

Die Nutzeroberfläche ist ohne Registrierung nutzbar. Der Fokus liegt darauf, schnell einen Überblick
über Buchungen (Zeiträume) zu bekommen und Details zu sehen.

Schriftliche Skizze der Nutzerreise:

1. **Startseite (Home)**  
   Kurzer Überblick, wie das Angebot funktioniert, und ein direkter Einstieg in den Katalog.
2. **Katalog (Buchungen-Liste)**  
   Paginierte Liste aller Buchungen. Jede Zeile zeigt den Artikelnamen, ein Thumbnail (falls vorhanden)
   und die Zeitspanne.
3. **Buchung-Detailansicht**  
   Detailinformationen zu Artikel und Station sowie eine visuelle Darstellung der Zeitspanne im Kalender.
4. **Informationsseiten**  
   FAQ, Über Uns und Kontakt vermitteln Hintergrund, Regeln und Ansprechpersonen.
