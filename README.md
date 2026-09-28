# AI Clipboard manager
### School Project


## Plan

### Het project is opgezet in ratatui, en het primaire opzet is:
* Project scaffolden, opzetten van packages zoals `ratatui`, `arboard`, `serde`, `serde_json`, en `dirs`.
* Categorie implementatie door gebruik van een `Category::detect` functie die data bijhoudt (een watcher) en automatisch organiseert.
* De watcher moet elke 400ms alleen nieuwe data ophalen, zodat alle data niet opnieuw wordt afgelezen. Dit is minder efficient.
* De UI gaat ratatui componenten gebruiken die vooral op low-level werken. Veel states en functies moeten daarom zelf behandeld worden in deze framework.

