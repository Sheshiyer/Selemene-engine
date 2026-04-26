#!/usr/bin/env python3
"""
Validate Agent 35 Implementation
- Verify all 9 planets have qualities
- Check description word counts (>50 words)
- Validate data completeness
"""

import re

# Extract descriptions from wisdom_data.rs
with open('src/wisdom_data.rs', 'r') as f:
    content = f.read()

# Find all planet descriptions
planet_pattern = r'VedicPlanet::(\w+),\s+PlanetaryPeriodQualities\s+{.*?description:\s+"([^"]+)"'
matches = re.findall(planet_pattern, content, re.DOTALL)

print("=" * 70)
print("AGENT 35 VALIDATION REPORT")
print("=" * 70)
print()

planets_found = []
descriptions = {}

for planet, desc in matches:
    planets_found.append(planet)
    word_count = len(desc.split())
    descriptions[planet] = {
        'description': desc,
        'word_count': word_count,
        'valid': word_count > 50
    }

# Expected planets
expected_planets = ['Sun', 'Moon', 'Mars', 'Mercury', 'Jupiter', 'Venus', 'Saturn', 'Rahu', 'Ketu']

print(f"✓ Planets Found: {len(planets_found)}/9")
print()

# Check each planet
all_valid = True
for planet in expected_planets:
    if planet in descriptions:
        data = descriptions[planet]
        status = "✅" if data['valid'] else "❌"
        print(f"{status} {planet:10} - {data['word_count']:3} words {'(PASS)' if data['valid'] else '(FAIL: <50 words)'}")
        if not data['valid']:
            all_valid = False
    else:
        print(f"❌ {planet:10} - MISSING")
        all_valid = False

print()
print("=" * 70)

# Verify themes, life_areas, challenges, opportunities
theme_pattern = r'themes:\s+vec!\[(.*?)\]'
life_areas_pattern = r'life_areas:\s+vec!\[(.*?)\]'
challenges_pattern = r'challenges:\s+vec!\[(.*?)\]'
opportunities_pattern = r'opportunities:\s+vec!\[(.*?)\]'

def count_items(pattern, content):
    matches = re.findall(pattern, content, re.DOTALL)
    if matches:
        # Count string literals
        return len(re.findall(r'\.to_string\(\)', matches[0]))
    return 0

print("DATA COMPLETENESS CHECK")
print("=" * 70)

# Check one planet in detail (Sun)
sun_section = content[content.find('VedicPlanet::Sun,'):content.find('VedicPlanet::Moon,')]

themes_count = count_items(theme_pattern, sun_section)
life_areas_count = count_items(life_areas_pattern, sun_section)
challenges_count = count_items(challenges_pattern, sun_section)
opportunities_count = count_items(opportunities_pattern, sun_section)

print(f"Sun Planet Validation:")
print(f"  Themes:        {themes_count} items")
print(f"  Life Areas:    {life_areas_count} items")
print(f"  Challenges:    {challenges_count} items")
print(f"  Opportunities: {opportunities_count} items")
print()

# Check witness.rs exists and has tests
try:
    with open('src/witness.rs', 'r') as f:
        witness_content = f.read()
    
    test_count = len(re.findall(r'#\[test\]', witness_content))
    print(f"✓ Witness module exists")
    print(f"✓ Unit tests found: {test_count}")
    
    # Check for consciousness level functions
    has_beginner = 'generate_beginner_prompt' in witness_content
    has_intermediate = 'generate_intermediate_prompt' in witness_content
    has_advanced = 'generate_advanced_prompt' in witness_content
    
    print(f"{'✓' if has_beginner else '✗'} Beginner prompt function")
    print(f"{'✓' if has_intermediate else '✗'} Intermediate prompt function")
    print(f"{'✓' if has_advanced else '✗'} Advanced prompt function")
    
except FileNotFoundError:
    print("✗ witness.rs NOT FOUND")
    all_valid = False

print()
print("=" * 70)

if all_valid and len(planets_found) == 9:
    print("✅ ALL VALIDATION CHECKS PASSED")
else:
    print("❌ VALIDATION FAILED - See errors above")

print("=" * 70)
