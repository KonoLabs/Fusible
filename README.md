# Fusible

> **Le fusible de votre firmware.** Un linter électronique en Rust : il lit le
> Circuit JSON généré par [tscircuit](https://tscircuit.com), vérifie les
> règles électriques et physiques du circuit — et si une règle échoue, **le
> fusible saute** : le processus sort en erreur et la génération du code
> firmware n'a jamais lieu.

Plutôt que de laisser un circuit cassé atteindre le firmware, Fusible remonte
des diagnostics façon compilateur Rust (`miette`), avec la ligne fautive
surlignée dans le fichier source.

## État du projet

| Phase | Contenu | État |
|---|---|---|
| 1 (MVP) | CLI, parser Circuit JSON, moteur de règles, diagnostics | ✅ livré |
| 2 | `mcu::` — validation du pin-muxing contre la cible silicium | 🔜 |
| 2 | `codegen::` — génération de `pins.rs` pour Embassy (si fusible intact) | 🔜 |

## Installation

```sh
cargo install --path .
# ou télécharger une archive depuis les releases GitHub
```

## Utilisation

```sh
fusible --input circuit.json --mcu stm32g431
```

Circuit valide :

```
✅ Circuit validé (Fusible intact)
```

Règle violée (exit code 1) :

```
× le fusible a sauté : 1 violation(s) électrique(s)
× [unconnected_net] le net « I2C_SDA » n'a que 1 connexion(s) (minimum 2) : il est flottant
  help: chaque net doit relier au moins deux pads/ports — complète le routage dans tscircuit
```

JSON malformé :

```
× Circuit JSON invalide : expected `,` or `}` at line 4 column 1
   ╭─[circuit.json:4:1]
 4 │ ]
   · ┬
   · ╰── la désérialisation a échoué ici
```

Deux exemples sont fournis dans [`examples/`](examples/) : `circuit.json`
(fusible intact) et `circuit-flottant.json` (fusible sauté).

## Architecture

```
src/
├── main.rs                 # Lire → Parser → Linter → verdict
├── cli.rs                  # clap : --input, --mcu
├── parser.rs               # enum Serde à tag interne + AST Circuit
│   └── diagnostic.rs       # rendu miette des erreurs de parsing
├── linter.rs               # trait Rule + Linter + règle unconnected_net
│   ├── diagnostics.rs      # Diagnostic / FuseBlown (miette)
│   └── tests.rs
```

Le Circuit JSON de tscircuit est un tableau d'objets hétérogènes discriminés
par un champ `type` : Serde le désérialise via un enum à tag interne, et les
types non modélisés sont *ignorés* plutôt que rejetés — le fusible ne saute
que pour de vraies violations électriques.

## Développement

```sh
mise install        # installe lefthook (conventional commits)
cargo test          # suite de tests
cargo clippy --all-targets -- -D warnings
filet check .       # qualité (https://github.com/FacileStudio/filet)
```

## Licence

[MIT](LICENSE)
