#!/usr/bin/env pike
// 提取 txpike9 所有房间数据

#define ROOT "/usr/local/games/txpike9"
#define ROOM_DIR ROOT "/gamenv/d"
#define OUTPUT_FILE ROOT "/rooms_data.json"

void extract_rooms() {
    mapping all_rooms = ([]);
    int room_count = 0;

    // 获取所有区域目录
    array(string) areas = get_dir(ROOM_DIR);

    werror("发现 %d 个区域\n", sizeof(areas));

    foreach(areas, string area) {
        // 跳过非目录文件（检查是否包含.pike后缀）
        if(search(area, ".pike") != -1) continue;
        if(search(area, ".") == 0 && area != ".") continue;

        string area_path = ROOM_DIR + "/" + area;

        // 尝试获取该区域下的所有房间
        array(string) rooms = get_dir(area_path);
        if(!rooms) continue;

        foreach(rooms, string room_name) {
            if(search(room_name, ".pike") != -1) continue;
            if(search(room_name, ".") == 0 && room_name != ".") continue;

            string room_file = area_path + "/" + room_name;

            // 读取房间文件内容
            string content = Stdio.read_file(room_file);
            if(!content) continue;

            mapping room_data = parse_room(content, area, room_name);
            if(room_data && room_data["name_cn"]) {
                string room_id = area + "/" + room_name;
                all_rooms[room_id] = room_data;
                room_count++;

                // 每100个房间输出一次进度
                if(room_count % 100 == 0) {
                    werror("已提取 %d 个房间...\n", room_count);
                }
            }
        }
    }

    werror("总共提取了 %d 个房间\n", room_count);

    // 输出为JSON
    string json = Standards.JSON.encode(all_rooms);
    Stdio.write_file(OUTPUT_FILE, json);
    werror("房间数据已保存到: %s\n", OUTPUT_FILE);
}

mapping parse_room(string content, string area, string room_name) {
    mapping room = ([]);
    array(string) lines = content / "\n";

    room["id"] = area + "/" + room_name;
    room["area"] = area;
    room["name"] = room_name;

    // 解析基本属性
    array(string) exits = ({});
    array(string) npcs = ({});
    string links = "";

    foreach(lines, string line) {
        line = String.trim_all_whites(line);

        // 解析 name_cn
        string name_cn;
        if(sscanf(line, "name_cn=\"%s\"", name_cn) == 1 ||
           sscanf(line, "name_cn=\"%s\";", name_cn) == 1) {
            room["name_cn"] = name_cn;
        }

        // 解析 desc
        string desc;
        if(sscanf(line, "desc=\"%s\"", desc) == 1) {
            room["desc"] = desc;
        }
        else if(sscanf(line, "desc+=\"%s\"", string desc_add) == 1) {
            if(!room["desc"]) room["desc"] = "";
            room["desc"] += desc_add;
        }

        // 解析 exits
        if(has_prefix(line, "exits[")) {
            string dir, target;
            if(sscanf(line, "exits[\"%s\"]=ROOT \"gamenv/d/%s\";", dir, target) == 2) {
                exits += ({dir + ":" + target});
            }
            else if(sscanf(line, "exits[\"%s\"]=ROOT \"gamenv/d/%s\"", dir, target) == 2) {
                exits += ({dir + ":" + target});
            }
        }

        // 解析NPC
        if(has_prefix(line, "add_items(({ROOT \"gamenv/clone/npc/")) {
            string npc_path;
            string pattern = "add_items(({ROOT \"gamenv/clone/npc/" + "%s\"}))";
            if(sscanf(line, pattern, npc_path) == 1) {
                npcs += ({npc_path});
            }
        }

        // 解析 links
        if(has_prefix(line, "links=")) {
            string link_content;
            if(sscanf(line, "links=\"%s\"", link_content) == 1) {
                links = link_content;
            }
        }
        else if(has_prefix(line, "links+=\"")) {
            string link_add;
            if(sscanf(line, "links+=\"%s\"", link_add) == 1) {
                links += link_add;
            }
        }

        // 检测房间类型
        if(search(line, "inherit WAPMUD_STORE") != -1) {
            room["room_type"] = "store";
        }
        else if(search(line, "inherit WAPMUD_ROOM") != -1) {
            if(!room["room_type"]) room["room_type"] = "normal";
        }

        // 检测属性
        if(search(line, "int is_peaceful()") != -1) {
            if(search(content, "return 1;") > search(line, "int is_peaceful()")) {
                room["is_peaceful"] = 1;
            }
        }
        if(search(line, "int is_bedroom()") != -1) {
            if(search(content, "return 1;") > search(line, "int is_bedroom()")) {
                room["is_bedroom"] = 1;
            }
        }
    }

    if(sizeof(exits) > 0) {
        room["exits"] = exits;
    }
    if(sizeof(npcs) > 0) {
        room["npcs"] = npcs;
    }
    if(links != "") {
        room["links"] = links;
    }

    return room;
}

int has_prefix(string s, string prefix) {
    return sscanf(s, "%" + sizeof(prefix) + "s", prefix) == 1;
}

int main(int argc, array(string) argv) {
    werror("开始提取房间数据...\n");
    extract_rooms();
    return 0;
}
