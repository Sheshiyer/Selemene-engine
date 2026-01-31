# Wisdom Data Quick Reference Guide

> Practical code examples for accessing and using WitnessOS wisdom data

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Loading Data Files](#loading-data-files)
3. [Common Query Patterns](#common-query-patterns)
4. [Cross-System Lookups](#cross-system-lookups)
5. [Integration Examples](#integration-examples)
6. [API Patterns](#api-patterns)
7. [Performance Tips](#performance-tips)

---

## Getting Started

### Directory Structure

```
wisdom-references/
├── astrology/          # Vedic astrology data
├── human_design/       # HD system (12 files)
├── iching/             # I Ching hexagrams
├── tarot/              # Tarot cards
├── sacred_geometry/    # Geometric symbols
├── enneagram/          # Personality types
├── gene_keys/          # Gene Keys archetypes
└── *.json              # Root-level files (TCM, biofield, etc.)
```

### Basic Python Setup

```python
import json
from pathlib import Path

# Base path to wisdom data
WISDOM_PATH = Path("wisdom-references")

def load_wisdom_data(filepath):
    """Universal data loader with error handling"""
    try:
        with open(WISDOM_PATH / filepath, 'r', encoding='utf-8') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"File not found: {filepath}")
        return None
    except json.JSONDecodeError as e:
        print(f"JSON error in {filepath}: {e}")
        return None
```

---

## Loading Data Files

### Load Human Design Gates

```python
# Load gates data
gates = load_wisdom_data("human_design/gates.json")

# Access specific gate
gate_1 = gates["1"]
print(f"Gate {gate_1['number']}: {gate_1['hd_name']}")
print(f"Center: {gate_1['center']}")
print(f"Shadow → Gift → Siddhi: {gate_1['shadow']} → {gate_1['gift']} → {gate_1['siddhi']}")

# Output:
# Gate 1: Self-Expression
# Center: G (Identity)
# Shadow → Gift → Siddhi: Entropy → Freshness → Beauty
```

### Load I Ching Hexagrams

```python
# Load hexagram data (use complete version for line meanings)
hexagrams = load_wisdom_data("iching/hexagrams_complete.json")

# Get hexagram by number
hex_1 = hexagrams["hexagrams"]["1"]
print(f"{hex_1['chinese_name']} - {hex_1['english_name']}")
print(f"Judgment: {hex_1['judgment'][:100]}...")

# Access changing line
line_3 = hex_1["lines"]["3"]
print(f"Line 3: {line_3['text']}")
print(f"Transforms to hexagram: {line_3['changing_to']}")
```

### Load Tarot Cards

```python
# Load Major Arcana
tarot = load_wisdom_data("tarot/major_arcana.json")

# Get specific card
fool = tarot["cards"][0]  # Array, so use index
print(f"{fool['name']} - {fool['archetype']}")
print(f"Upright: {fool['upright_meaning']}")
print(f"Astrological: {fool['astrological_correspondence']}")

# Iterate through all cards
for card in tarot["cards"]:
    print(f"{card['number']}: {card['name']} - {card['keywords']}")
```

### Load Enneagram Types

```python
# Load Enneagram data
enneagram = load_wisdom_data("enneagram/types.json")

# Access type 1
type_1 = enneagram["types"]["1"]
print(f"Type {type_1['name']}")
print(f"Core Fear: {type_1['core_fear']}")
print(f"Integration Arrow: {type_1['integration_arrow']}")

# Get wing variations
wings = type_1["wings"]
print(f"Wing possibilities: {', '.join(wings)}")
```

### Load TCM Organ Clock

```python
# Load organ clock
tcm_clock = load_wisdom_data("tcm_organ_clock.json")

# Find organ active at specific time
def get_active_organ(hour):
    """Get organ active at given hour (0-23)"""
    for organ_data in tcm_clock["organ_clock"]:
        time_range = organ_data["time_range"]
        start, end = time_range.split("-")
        start_hour = int(start.split(":")[0])
        end_hour = int(end.split(":")[0])
        
        if start_hour <= hour < end_hour:
            return organ_data
    return None

# Example: What's active at 3 AM?
organ = get_active_organ(3)
print(f"Active organ: {organ['organ']}")
print(f"Element: {organ['element']}")
print(f"Peak energy: {organ['peak_energy']}")
```

---

## Common Query Patterns

### Query 1: Find Gates by Center

```python
gates = load_wisdom_data("human_design/gates.json")

def gates_by_center(center_name):
    """Get all gates associated with a center"""
    return {
        gate_num: gate_data 
        for gate_num, gate_data in gates.items()
        if gate_data.get("center") == center_name
    }

# Example: All Throat gates
throat_gates = gates_by_center("Throat")
print(f"Throat gates: {list(throat_gates.keys())}")
# Output: Throat gates: ['8', '12', '16', '20', '23', '31', '33', '35', '45', '56', '62']
```

### Query 2: Find Hexagrams by Trigram

```python
hexagrams = load_wisdom_data("iching/hexagrams.json")

def find_by_trigram(trigram_name, position="upper"):
    """Find hexagrams with specific trigram"""
    results = []
    for num, hex_data in hexagrams["hexagrams"].items():
        if hex_data["trigrams"][position]["name"] == trigram_name:
            results.append({
                "number": num,
                "name": hex_data["english_name"],
                "chinese": hex_data["chinese_name"]
            })
    return results

# Example: All hexagrams with "Heaven" as upper trigram
heaven_hexagrams = find_by_trigram("Heaven", "upper")
for h in heaven_hexagrams:
    print(f"{h['number']}: {h['chinese']} - {h['name']}")
```

### Query 3: Lookup Nakshatra by Degree

```python
nakshatras = load_wisdom_data("astrology/nakshatras.json")

def find_nakshatra_by_degree(degrees):
    """Find nakshatra for given zodiacal degree (0-360)"""
    # Each nakshatra spans 13°20' (13.333 degrees)
    nakshatra_span = 13.333333
    nakshatra_index = int(degrees // nakshatra_span)
    
    return nakshatras["nakshatras"][nakshatra_index]

# Example: Moon at 157 degrees (7° Virgo)
moon_nakshatra = find_nakshatra_by_degree(157)
print(f"Nakshatra: {moon_nakshatra['name']}")
print(f"Ruling planet: {moon_nakshatra['ruling_planet']}")
print(f"Symbol: {moon_nakshatra['symbol']}")
```

### Query 4: Get Tarot Card by Name or Number

```python
tarot = load_wisdom_data("tarot/rider_waite.json")

def get_tarot_card(identifier):
    """Get card by number or name"""
    # Search Major Arcana
    for card in tarot["major_arcana"]:
        if card["number"] == identifier or card["name"].lower() == str(identifier).lower():
            return card
    
    # Search Minor Arcana
    for suit, suit_data in tarot["minor_arcana"]["suits"].items():
        for card in suit_data["cards"]:
            if card["name"].lower() == str(identifier).lower():
                return card
    
    return None

# Examples
fool = get_tarot_card(0)
tower = get_tarot_card("The Tower")
ace_wands = get_tarot_card("Ace of Wands")
```

### Query 5: Calculate Vimshottari Dasha

```python
import datetime

vimshottari = load_wisdom_data("vimshottari_periods.json")

def calculate_current_dasha(birth_date, moon_nakshatra_index):
    """Calculate current planetary period"""
    sequence = vimshottari["planetary_sequence"]
    durations = vimshottari["period_durations"]
    
    # Start dasha is determined by moon's nakshatra
    start_planet_index = moon_nakshatra_index % 9
    
    # Calculate years since birth
    today = datetime.date.today()
    age_years = (today - birth_date).days / 365.25
    
    # Find current dasha
    accumulated_years = 0
    current_index = start_planet_index
    
    for _ in range(9):  # Max 9 planets in sequence
        planet = sequence[current_index]
        planet_years = durations[planet]
        
        if accumulated_years + planet_years > age_years:
            years_into_dasha = age_years - accumulated_years
            return {
                "planet": planet,
                "years_total": planet_years,
                "years_remaining": planet_years - years_into_dasha
            }
        
        accumulated_years += planet_years
        current_index = (current_index + 1) % 9
    
    return None

# Example
birth = datetime.date(1990, 5, 15)
moon_nak = 10  # Maghā
dasha = calculate_current_dasha(birth, moon_nak)
print(f"Current Dasha: {dasha['planet']}")
print(f"Years remaining: {dasha['years_remaining']:.1f}")
```

---

## Cross-System Lookups

### Lookup 1: I Ching → Human Design → Gene Keys

```python
# Load all three systems
iching = load_wisdom_data("iching/hexagrams.json")
hd_gates = load_wisdom_data("human_design/gates.json")
gene_keys = load_wisdom_data("gene_keys/archetypes.json")

def cross_system_lookup(hexagram_number):
    """Get interpretations across all three systems"""
    hex_num = str(hexagram_number)
    
    return {
        "iching": {
            "name": iching["hexagrams"][hex_num]["english_name"],
            "chinese": iching["hexagrams"][hex_num]["chinese_name"],
            "judgment": iching["hexagrams"][hex_num]["judgment"]
        },
        "human_design": {
            "gate_name": hd_gates[hex_num]["hd_name"],
            "center": hd_gates[hex_num]["center"],
            "channel": hd_gates[hex_num]["channel_pairing"]
        },
        "gene_keys": {
            "name": gene_keys["keys"][hex_num]["name"],
            "shadow": gene_keys["keys"][hex_num]["shadow"]["name"],
            "gift": gene_keys["keys"][hex_num]["gift"]["name"],
            "siddhi": gene_keys["keys"][hex_num]["siddhi"]["name"]
        }
    }

# Example: Gate/Hexagram 1 across all systems
cross_ref = cross_system_lookup(1)
print(json.dumps(cross_ref, indent=2))
```

### Lookup 2: Vedic + TCM Face Reading Integration

```python
vedic_face = load_wisdom_data("vedic_face_correlations.json")
tcm_face = load_wisdom_data("tcm_face_correlations.json")
integration = load_wisdom_data("vedic_tcm_correspondences.json")

def integrated_face_reading(face_region):
    """Get both Vedic and TCM interpretations"""
    vedic_interp = vedic_face["features"].get(face_region, {})
    tcm_interp = tcm_face["tcm_face_map"].get(face_region, {})
    
    return {
        "region": face_region,
        "vedic": vedic_interp,
        "tcm": tcm_interp,
        "integration_notes": integration.get(face_region, "See correspondences")
    }

# Example: Forehead reading
forehead_reading = integrated_face_reading("forehead")
```

### Lookup 3: Planetary Correspondences Across Systems

```python
def planetary_synthesis(planet_name):
    """Get planet info across astrology, HD, and TCM"""
    astro_planets = load_wisdom_data("astrology/planets.json")
    hd_activations = load_wisdom_data("human_design/planetary_activations.json")
    vedic_tcm = load_wisdom_data("vedic_tcm_correspondences.json")
    
    # Find planetary data
    planet = next((p for p in astro_planets["planets"] if p["name"] == planet_name), None)
    planet_organs = vedic_tcm["planetary_organ_map"].get(planet_name, [])
    
    return {
        "vedic_astrology": planet,
        "organs": planet_organs,
        "human_design_role": f"Activates gates in personality/design"
    }

# Example
jupiter_data = planetary_synthesis("Jupiter")
```

---

## Integration Examples

### Example 1: Generate Complete Human Design Chart Snippet

```python
def mini_hd_chart(birth_gates):
    """Generate HD chart summary from activated gates
    
    Args:
        birth_gates: List of gate numbers (e.g., [1, 13, 25, 51])
    """
    gates_data = load_wisdom_data("human_design/gates.json")
    centers_data = load_wisdom_data("human_design/centers.json")
    channels_data = load_wisdom_data("human_design/channels.json")
    
    # Determine defined centers
    defined_centers = set()
    for gate_num in birth_gates:
        gate = gates_data[str(gate_num)]
        defined_centers.add(gate["center"])
    
    # Check for channels (need both gates of a channel)
    active_channels = []
    for channel in channels_data["channels"]:
        gate_nums = [int(g) for g in channel["name"].split("-")]
        if all(g in birth_gates for g in gate_nums):
            active_channels.append(channel)
    
    return {
        "activated_gates": len(birth_gates),
        "defined_centers": list(defined_centers),
        "channels": [c["name"] for c in active_channels],
        "definition_count": len(active_channels)
    }

# Example
chart = mini_hd_chart([1, 8, 13, 25, 51])
print(f"Defined Centers: {chart['defined_centers']}")
print(f"Channels: {chart['channels']}")
```

### Example 2: Daily Wisdom Integration

```python
import datetime

def daily_wisdom_synthesis():
    """Get today's wisdom across multiple systems"""
    today = datetime.date.today()
    hour = datetime.datetime.now().hour
    
    # TCM organ active now
    tcm_clock = load_wisdom_data("tcm_organ_clock.json")
    current_organ = get_active_organ(hour)
    
    # I Ching hexagram of the day (simple date-based)
    hexagrams = load_wisdom_data("iching/hexagrams.json")
    hex_of_day = ((today.year + today.month + today.day) % 64) + 1
    todays_hexagram = hexagrams["hexagrams"][str(hex_of_day)]
    
    # Consciousness practice for time
    practices = load_wisdom_data("consciousness_practices.json")
    # (Find practice matching current hour)
    
    return {
        "date": str(today),
        "time": f"{hour:02d}:00",
        "tcm_organ": {
            "name": current_organ["organ"],
            "element": current_organ["element"],
            "practice": current_organ["practices"]["optimal"]
        },
        "hexagram": {
            "number": hex_of_day,
            "name": todays_hexagram["english_name"],
            "guidance": todays_hexagram["judgment"][:100]
        }
    }

# Get today's wisdom
wisdom = daily_wisdom_synthesis()
print(json.dumps(wisdom, indent=2))
```

### Example 3: Archetypal Profile Builder

```python
def build_archetypal_profile(birth_data):
    """Create multi-system archetypal profile
    
    Args:
        birth_data: Dict with keys like 'enneagram_type', 'sun_gate', etc.
    """
    enneagram = load_wisdom_data("enneagram/types.json")
    gates = load_wisdom_data("human_design/gates.json")
    tarot = load_wisdom_data("tarot/major_arcana.json")
    
    profile = {
        "enneagram": enneagram["types"][str(birth_data["enneagram_type"])],
        "sun_gate": gates[str(birth_data["sun_gate"])],
        "life_card": tarot["cards"][birth_data["life_path_number"]]
    }
    
    # Synthesize keywords
    keywords = []
    keywords.extend(profile["enneagram"].get("healthy_traits", [])[:3])
    keywords.append(profile["sun_gate"]["gift"])
    keywords.extend(profile["life_card"]["keywords"][:2])
    
    profile["synthesis"] = {
        "keywords": keywords,
        "core_theme": f"{profile['enneagram']['name']} expressing through {profile['sun_gate']['hd_name']}"
    }
    
    return profile

# Example
person_profile = build_archetypal_profile({
    "enneagram_type": 4,
    "sun_gate": 1,
    "life_path_number": 7
})
```

---

## API Patterns

### RESTful Endpoint Examples

```python
from flask import Flask, jsonify, request
app = Flask(__name__)

# Cache loaded data
WISDOM_CACHE = {}

def get_cached_data(filepath):
    """Load and cache wisdom data"""
    if filepath not in WISDOM_CACHE:
        WISDOM_CACHE[filepath] = load_wisdom_data(filepath)
    return WISDOM_CACHE[filepath]

# Endpoint: Get Human Design gate
@app.route('/api/hd/gate/<int:gate_num>', methods=['GET'])
def get_hd_gate(gate_num):
    gates = get_cached_data("human_design/gates.json")
    gate = gates.get(str(gate_num))
    if gate:
        return jsonify(gate)
    return jsonify({"error": "Gate not found"}), 404

# Endpoint: Get I Ching reading
@app.route('/api/iching/cast', methods=['POST'])
def cast_iching():
    # Generate random hexagram or use provided method
    import random
    hex_num = random.randint(1, 64)
    
    hexagrams = get_cached_data("iching/hexagrams_complete.json")
    reading = hexagrams["hexagrams"][str(hex_num)]
    
    return jsonify({
        "hexagram": reading,
        "timestamp": datetime.datetime.now().isoformat()
    })

# Endpoint: Get TCM organ for current time
@app.route('/api/tcm/current-organ', methods=['GET'])
def current_tcm_organ():
    hour = datetime.datetime.now().hour
    organ = get_active_organ(hour)
    return jsonify(organ)

# Endpoint: Search across systems
@app.route('/api/search', methods=['GET'])
def search_wisdom():
    query = request.args.get('q', '').lower()
    results = []
    
    # Search gates
    gates = get_cached_data("human_design/gates.json")
    for num, gate in gates.items():
        if query in gate.get("hd_name", "").lower():
            results.append({"type": "hd_gate", "data": gate})
    
    # Search hexagrams
    hexagrams = get_cached_data("iching/hexagrams.json")
    for num, hex in hexagrams["hexagrams"].items():
        if query in hex.get("english_name", "").lower():
            results.append({"type": "hexagram", "data": hex})
    
    return jsonify({"query": query, "results": results[:10]})
```

### GraphQL Schema Example

```python
import graphene

class HumanDesignGate(graphene.ObjectType):
    number = graphene.Int()
    hd_name = graphene.String()
    center = graphene.String()
    shadow = graphene.String()
    gift = graphene.String()
    siddhi = graphene.String()

class Query(graphene.ObjectType):
    gate = graphene.Field(HumanDesignGate, number=graphene.Int())
    gates_by_center = graphene.List(HumanDesignGate, center=graphene.String())
    
    def resolve_gate(self, info, number):
        gates = get_cached_data("human_design/gates.json")
        gate_data = gates.get(str(number))
        if gate_data:
            return HumanDesignGate(**gate_data)
        return None
    
    def resolve_gates_by_center(self, info, center):
        gates = get_cached_data("human_design/gates.json")
        results = [
            HumanDesignGate(number=int(num), **data)
            for num, data in gates.items()
            if data.get("center") == center
        ]
        return results

schema = graphene.Schema(query=Query)

# Example query:
# {
#   gate(number: 1) {
#     hdName
#     center
#     gift
#   }
# }
```

---

## Performance Tips

### Tip 1: Cache Loaded Data

```python
import functools

@functools.lru_cache(maxsize=50)
def load_wisdom_cached(filepath):
    """Cached version of data loader"""
    return load_wisdom_data(filepath)

# Use cached version for repeated access
gates = load_wisdom_cached("human_design/gates.json")
```

### Tip 2: Index Creation for Fast Lookups

```python
def create_gate_index():
    """Build lookup indices for faster queries"""
    gates = load_wisdom_data("human_design/gates.json")
    
    indices = {
        "by_center": {},
        "by_circuit": {},
        "by_channel": {}
    }
    
    for num, gate in gates.items():
        # Index by center
        center = gate["center"]
        if center not in indices["by_center"]:
            indices["by_center"][center] = []
        indices["by_center"][center].append(num)
        
        # Index by circuit
        circuit = gate.get("circuit")
        if circuit and circuit not in indices["by_circuit"]:
            indices["by_circuit"][circuit] = []
        if circuit:
            indices["by_circuit"][circuit].append(num)
    
    return indices

# Build once, query many times
GATE_INDEX = create_gate_index()

# Fast lookup
throat_gates = GATE_INDEX["by_center"]["Throat"]
```

### Tip 3: Lazy Loading for Large Datasets

```python
class WisdomDataManager:
    """Lazy-loading wisdom data manager"""
    
    def __init__(self):
        self._cache = {}
    
    @property
    def gates(self):
        if "gates" not in self._cache:
            self._cache["gates"] = load_wisdom_data("human_design/gates.json")
        return self._cache["gates"]
    
    @property
    def hexagrams(self):
        if "hexagrams" not in self._cache:
            self._cache["hexagrams"] = load_wisdom_data("iching/hexagrams_complete.json")
        return self._cache["hexagrams"]
    
    def clear_cache(self):
        """Clear all cached data"""
        self._cache.clear()

# Usage
wisdom = WisdomDataManager()
gate_1 = wisdom.gates["1"]  # Loads on first access
hex_1 = wisdom.hexagrams["hexagrams"]["1"]  # Loads on first access
```

### Tip 4: Batch Processing

```python
def batch_gate_lookup(gate_numbers):
    """Efficiently lookup multiple gates"""
    gates = load_wisdom_cached("human_design/gates.json")
    
    return [
        {
            "number": num,
            "data": gates.get(str(num))
        }
        for num in gate_numbers
    ]

# Get multiple gates in one call
my_gates = batch_gate_lookup([1, 13, 25, 51])
```

---

## Advanced Patterns

### Pattern 1: Semantic Search with Embeddings

```python
from sentence_transformers import SentenceTransformer
import numpy as np

# Load model (do once)
model = SentenceTransformer('all-MiniLM-L6-v2')

def build_semantic_index():
    """Create embeddings for all wisdom texts"""
    gates = load_wisdom_data("human_design/gates.json")
    
    texts = []
    metadata = []
    
    for num, gate in gates.items():
        text = f"{gate['hd_name']} {gate.get('keynote', '')} {gate.get('gift', '')}"
        texts.append(text)
        metadata.append({"type": "gate", "number": num})
    
    embeddings = model.encode(texts)
    
    return {
        "embeddings": embeddings,
        "metadata": metadata,
        "texts": texts
    }

def semantic_search(query, index, top_k=5):
    """Find most relevant wisdom data by semantic similarity"""
    query_embedding = model.encode([query])
    
    # Cosine similarity
    similarities = np.dot(index["embeddings"], query_embedding.T).flatten()
    top_indices = np.argsort(similarities)[-top_k:][::-1]
    
    return [
        {
            "text": index["texts"][i],
            "metadata": index["metadata"][i],
            "score": similarities[i]
        }
        for i in top_indices
    ]

# Build index once
# semantic_index = build_semantic_index()

# Search
# results = semantic_search("creative expression and identity", semantic_index)
```

### Pattern 2: Real-Time Wisdom Feed

```python
import time

def wisdom_stream(duration_seconds=60):
    """Stream wisdom updates in real-time"""
    start_time = time.time()
    
    while time.time() - start_time < duration_seconds:
        current_hour = datetime.datetime.now().hour
        
        # Get current TCM organ
        organ = get_active_organ(current_hour)
        
        # Get hexagram of the moment
        timestamp = int(time.time())
        hex_num = (timestamp % 64) + 1
        hexagrams = load_wisdom_cached("iching/hexagrams.json")
        hex_data = hexagrams["hexagrams"][str(hex_num)]
        
        yield {
            "timestamp": datetime.datetime.now().isoformat(),
            "tcm_organ": organ["organ"],
            "hexagram": hex_data["english_name"],
            "guidance": hex_data["judgment"][:100]
        }
        
        time.sleep(60)  # Update every minute

# Usage
# for update in wisdom_stream(300):
#     print(json.dumps(update, indent=2))
```

---

## JavaScript/TypeScript Examples

### Load and Query in Node.js

```javascript
const fs = require('fs');
const path = require('path');

// Load wisdom data
function loadWisdomData(filepath) {
  const fullPath = path.join(__dirname, 'wisdom-references', filepath);
  const rawData = fs.readFileSync(fullPath, 'utf8');
  return JSON.parse(rawData);
}

// Get Human Design gate
const gates = loadWisdomData('human_design/gates.json');
const gate1 = gates['1'];
console.log(`Gate ${gate1.number}: ${gate1.hd_name}`);

// Find gates by center
function gatesByCenter(centerName) {
  return Object.entries(gates)
    .filter(([num, gate]) => gate.center === centerName)
    .map(([num, gate]) => ({ number: num, ...gate }));
}

const throatGates = gatesByCenter('Throat');
console.log(`Throat gates: ${throatGates.map(g => g.number).join(', ')}`);
```

### React Hook for Wisdom Data

```typescript
import { useState, useEffect } from 'react';

interface HumanDesignGate {
  number: number;
  hd_name: string;
  center: string;
  gift: string;
  shadow: string;
  siddhi: string;
}

function useHumanDesignGate(gateNumber: number) {
  const [gate, setGate] = useState<HumanDesignGate | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    fetch(`/wisdom-references/human_design/gates.json`)
      .then(res => res.json())
      .then(data => {
        setGate(data[gateNumber.toString()]);
        setLoading(false);
      })
      .catch(err => {
        setError(err.message);
        setLoading(false);
      });
  }, [gateNumber]);
  
  return { gate, loading, error };
}

// Usage in component
function GateDisplay({ gateNumber }: { gateNumber: number }) {
  const { gate, loading, error } = useHumanDesignGate(gateNumber);
  
  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!gate) return <div>Gate not found</div>;
  
  return (
    <div>
      <h2>Gate {gate.number}: {gate.hd_name}</h2>
      <p>Center: {gate.center}</p>
      <p>Shadow → Gift → Siddhi: {gate.shadow} → {gate.gift} → {gate.siddhi}</p>
    </div>
  );
}
```

---

## Command-Line Tools

### Bash/Shell Examples

```bash
#!/bin/bash
# wisdom-query.sh - Command-line wisdom data querying

WISDOM_DIR="wisdom-references"

# Get gate by number
get_gate() {
  local gate_num=$1
  jq ".\"$gate_num\"" "$WISDOM_DIR/human_design/gates.json"
}

# Find gates by center
gates_by_center() {
  local center=$1
  jq "to_entries | map(select(.value.center == \"$center\")) | from_entries" \
    "$WISDOM_DIR/human_design/gates.json"
}

# Get hexagram
get_hexagram() {
  local hex_num=$1
  jq ".hexagrams.\"$hex_num\"" "$WISDOM_DIR/iching/hexagrams.json"
}

# Get current TCM organ
current_organ() {
  local hour=$(date +%H)
  # ... parse and query tcm_organ_clock.json
}

# Usage:
# ./wisdom-query.sh get_gate 1
# ./wisdom-query.sh gates_by_center "Throat"
```

---

## Testing Patterns

### Unit Test Example

```python
import unittest

class TestWisdomData(unittest.TestCase):
    
    def setUp(self):
        """Load data before each test"""
        self.gates = load_wisdom_data("human_design/gates.json")
        self.hexagrams = load_wisdom_data("iching/hexagrams.json")
    
    def test_gates_structure(self):
        """Verify gates have required fields"""
        gate_1 = self.gates["1"]
        self.assertIn("hd_name", gate_1)
        self.assertIn("center", gate_1)
        self.assertIn("gift", gate_1)
    
    def test_hexagram_count(self):
        """Verify 64 hexagrams present"""
        self.assertEqual(len(self.hexagrams["hexagrams"]), 64)
    
    def test_cross_reference(self):
        """Verify I Ching and HD gates align"""
        for i in range(1, 65):
            self.assertIn(str(i), self.gates)
            self.assertIn(str(i), self.hexagrams["hexagrams"])

if __name__ == '__main__':
    unittest.main()
```

---

## Summary: Most Common Use Cases

| Use Case | Files Needed | Example Code |
|----------|--------------|--------------|
| HD Chart Generation | `human_design/*.json` | `mini_hd_chart()` |
| I Ching Reading | `iching/hexagrams_complete.json` | `cast_iching()` |
| Daily Organ Energy | `tcm_organ_clock.json` | `get_active_organ()` |
| Personality Profile | `enneagram/types.json` | `type_1 = enneagram["types"]["1"]` |
| Cross-System Lookup | Multiple files | `cross_system_lookup()` |
| Semantic Search | All JSON files + embeddings | `semantic_search()` |

---

**Last Updated**: 2026-01-26  
**Version**: 1.0.0  
**Status**: ✅ Production Ready
