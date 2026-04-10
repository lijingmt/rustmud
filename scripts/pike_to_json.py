#!/usr/bin/env python3
"""
Pike Room to JSON Converter
Reads txpike9 Pike room files and converts them to JSON for the Rust MUD
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Any, Optional, Set

class PikeRoomParser:
    """Parser for txpike9 Pike room files"""

    # Direction mappings
    DIRECTION_MAP = {
        "east": "east",
        "west": "west",
        "south": "south",
        "north": "north",
        "up": "up",
        "down": "down",
        "northeast": "northeast",
        "northwest": "northwest",
        "southeast": "southeast",
        "southwest": "southwest",
        "enter": "enter",
        "out": "out",
    }

    def __init__(self, txpike9_root: str):
        self.txpike9_root = txpike9_root
        self.room_dir = os.path.join(txpike9_root, "gamenv/d")

    def parse_room_file(self, file_path: str, zone: str) -> Optional[Dict[str, Any]]:
        """Parse a single Pike room file"""
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
            return None

        room = {
            "name_cn": "",
            "desc": "",
            "exits": [],
            "npcs": [],
            "items": [],
            "links": "",
            "is_peaceful": False,
            "is_bedroom": False,
            "room_type": "normal",
        }

        # Extract name_cn
        name_match = re.search(r'name_cn\s*=\s*"([^"]*)"', content)
        if name_match:
            room["name_cn"] = name_match.group(1)

        # Extract desc
        desc_match = re.search(r'desc\s*=\s*"([^"]*)"', content)
        if desc_match:
            room["desc"] = desc_match.group(1)

        # Extract exits - keep the original path format for consistency
        exit_pattern = r'exits\["([^"]+)"\]\s*=\s*ROOT\s*"gamenv/d/([^"]+)"'
        for match in re.finditer(exit_pattern, content):
            direction, target_path = match.groups()
            # Keep the original path format (e.g., "xinshoucun/dongjie")
            room["exits"].append(f"{direction}:{target_path}")

        # Extract NPCs from add_items
        npc_pattern = r'add_items\(\(\{ROOT\s*"gamenv/clone/npc/([^"]+)"\}\)\)'
        for match in re.finditer(npc_pattern, content):
            npc_path = match.group(1)
            # Use the full path format: zone/npc_id
            # For example: xinshoucun/cunmin instead of just cunmin
            npc_id = npc_path.split("/")[-1] if "/" in npc_path else npc_path
            # Prepend zone to create full path (e.g., xinshoucun/cunmin)
            full_npc_id = f"{zone}/{npc_id}"
            if full_npc_id not in room["npcs"]:
                room["npcs"].append(full_npc_id)

        # Extract links
        links_match = re.search(r'links\s*=\s*"([^"]*)"', content)
        if links_match:
            room["links"] = links_match.group(1).replace(r'\n', '\n')
        else:
            # Check for multi-line links
            links_pattern = r'links\+\s*=\s*"([^"]*)"'
            all_links = []
            for match in re.finditer(links_pattern, content):
                all_links.append(match.group(1).replace(r'\n', '\n'))
            if all_links:
                room["links"] = "\n".join(all_links)

        # Check for is_peaceful()
        room["is_peaceful"] = bool(re.search(r'int\s+is_peaceful\(\)', content))

        # Check for is_bedroom()
        room["is_bedroom"] = bool(re.search(r'int\s+is_bedroom\(\)', content))

        # Check for inheritance to determine room type
        if re.search(r'inherit\s+WAPMUD_BANK', content):
            room["room_type"] = "bank"
        elif re.search(r'inherit\s+WAPMUD_STORE', content):
            room["room_type"] = "store"
        elif re.search(r'is_pawnshop\(\)', content):
            room["room_type"] = "pawnshop"
        elif re.search(r'is_bedroom\(\)', content):
            room["room_type"] = "bedroom"

        return room

    def scan_zone(self, zone: str) -> Dict[str, Any]:
        """Scan all room files in a zone directory"""
        zone_path = os.path.join(self.room_dir, zone)
        if not os.path.exists(zone_path):
            print(f"Zone path not found: {zone_path}")
            return {}

        rooms = {}

        # List all files in the zone directory
        for filename in os.listdir(zone_path):
            file_path = os.path.join(zone_path, filename)

            # Skip directories and special files
            if os.path.isdir(file_path) or filename.startswith('.'):
                continue

            # Parse the room file
            room = self.parse_room_file(file_path, zone)
            if room:
                # Use path format for room_id (e.g., "xinshoucun/xinshoucunguangchang")
                room_id = f"{zone}/{filename}"
                rooms[room_id] = room

        return rooms

    def scan_all_zones(self) -> Dict[str, Any]:
        """Scan all zone directories and convert all rooms"""
        all_rooms = {}

        # Get all zone directories
        zones = []
        for entry in os.listdir(self.room_dir):
            zone_path = os.path.join(self.room_dir, entry)
            if os.path.isdir(zone_path) and not entry.startswith('.'):
                zones.append(entry)

        print(f"Found {len(zones)} zones")

        for zone in sorted(zones):
            print(f"Scanning zone: {zone}")
            zone_rooms = self.scan_zone(zone)
            print(f"  Found {len(zone_rooms)} rooms")
            all_rooms.update(zone_rooms)

        return all_rooms

    def convert_to_json(self, output_path: str):
        """Convert all rooms to JSON file"""
        print("Converting Pike rooms to JSON...")
        rooms = self.scan_all_zones()

        print(f"\nTotal rooms found: {len(rooms)}")

        # Write to JSON
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(rooms, f, ensure_ascii=False, indent=2)

        print(f"JSON written to: {output_path}")


def main():
    import sys

    # Default paths
    txpike9_root = "/usr/local/games/txpike9"
    output_path = "/usr/local/games/rust/data/world/rooms_data.json"

    # Allow command line overrides
    if len(sys.argv) > 1:
        txpike9_root = sys.argv[1]
    if len(sys.argv) > 2:
        output_path = sys.argv[2]

    print(f"Pike Room Converter")
    print(f"  txpike9 root: {txpike9_root}")
    print(f"  Output: {output_path}")
    print()

    parser = PikeRoomParser(txpike9_root)
    parser.convert_to_json(output_path)


if __name__ == "__main__":
    main()
