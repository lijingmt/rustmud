#!/usr/bin/env pike
// 提取 txpike9 所有NPC数据

#define ROOT "/usr/local/games/txpike9"
#define NPC_DIR ROOT "/gamenv/clone/npc"
#define OUTPUT_FILE ROOT "/npcs_data.json"

void extract_npcs() {
    mapping all_npcs = ([]);
    int npc_count = 0;

    // 获取所有NPC区域目录
    array(string) areas = get_dir(NPC_DIR);

    werror("发现 %d 个NPC区域\n", sizeof(areas));

    foreach(areas, string area) {
        // 跳过非目录文件
        if(search(area, ".pike") != -1) continue;
        if(search(area, ".") == 0 && area != ".") continue;

        string area_path = NPC_DIR + "/" + area;

        // 尝试获取该区域下的所有NPC
        array(string) npcs = get_dir(area_path);
        if(!npcs) continue;

        foreach(npcs, string npc_name) {
            if(search(npc_name, ".pike") != -1) continue;
            if(search(npc_name, ".") == 0 && npc_name != ".") continue;

            string npc_file = area_path + "/" + npc_name;

            // 读取NPC文件内容
            string content = Stdio.read_file(npc_file);
            if(!content) continue;

            mapping npc_data = parse_npc(content, area, npc_name);
            if(npc_data && npc_data["name_cn"]) {
                string npc_id = area + "/" + npc_name;
                all_npcs[npc_id] = npc_data;
                npc_count++;

                // 每100个NPC输出一次进度
                if(npc_count % 100 == 0) {
                    werror("已提取 %d 个NPC...\n", npc_count);
                }
            }
        }
    }

    werror("总共提取了 %d 个NPC\n", npc_count);

    // 输出为JSON
    string json = Standards.JSON.encode(all_npcs);
    Stdio.write_file(OUTPUT_FILE, json);
    werror("NPC数据已保存到: %s\n", OUTPUT_FILE);
}

mapping parse_npc(string content, string area, string npc_name) {
    mapping npc = ([]);
    array(string) lines = content / "\n";

    npc["id"] = area + "/" + npc_name;
    npc["area"] = area;
    npc["name"] = npc_name;

    // 默认值
    npc["level"] = 1;
    npc["hp"] = 100;
    npc["max_hp"] = 100;
    npc["attack"] = 10;
    npc["defense"] = 5;

    foreach(lines, string line) {
        line = String.trim_all_whites(line);

        // 解析 name_cn
        string name_cn;
        if(sscanf(line, "name_cn=\"%s\"", name_cn) == 1 ||
           sscanf(line, "name_cn=\"%s\";", name_cn) == 1) {
            npc["name_cn"] = name_cn;
        }

        // 解析 desc
        string desc;
        if(sscanf(line, "desc=\"%s\"", desc) == 1) {
            npc["desc"] = desc;
        }

        // 解析 gender
        string gender;
        if(sscanf(line, "gender=\"%s\"", gender) == 1) {
            npc["gender"] = gender;
        }

        // 解析 level
        int level;
        if(sscanf(line, "set_level(%d);", level) == 1) {
            npc["level"] = level;
        }

        // 解析 hp/max_hp
        int hp;
        if(sscanf(line, "hp_max=%d;", hp) == 1) {
            npc["max_hp"] = hp;
            npc["hp"] = hp;
        }

        // 解析 daoheng (修为)
        int daoheng;
        if(sscanf(line, "daoheng=%d;", daoheng) == 1) {
            npc["daoheng"] = daoheng;
        }

        // 解析装备
        if(search(line, "equip_list+=") != -1) {
            if(!npc["equip_list"]) npc["equip_list"] = ({});
            string equip_path;
            if(sscanf(line, "equip_list+=(\"%s\"", equip_path) == 1) {
                npc["equip_list"] += ({equip_path});
            }
        }
    }

    return npc;
}

int main(int argc, array(string) argv) {
    werror("开始提取NPC数据...\n");
    extract_npcs();
    return 0;
}
