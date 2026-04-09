#!/usr/bin/env pike
// 提取 txpike9 所有物品数据

#define ROOT "/usr/local/games/txpike9"
#define ITEM_DIR ROOT "/gamenv/clone/item"
#define OUTPUT_FILE ROOT "/items_data.json"

void extract_items() {
    mapping all_items = ([]);
    int item_count = 0;

    // 获取所有物品区域目录
    array(string) areas = get_dir(ITEM_DIR);

    werror("发现 %d 个物品区域\n", sizeof(areas));

    foreach(areas, string area) {
        // 跳过非目录文件
        if(search(area, ".pike") != -1) continue;
        if(search(area, ".") == 0 && area != ".") continue;

        string area_path = ITEM_DIR + "/" + area;

        // 尝试获取该区域下的所有物品
        array(string) items = get_dir(area_path);
        if(!items) continue;

        foreach(items, string item_name) {
            // 跳过带后缀的变体物品（Xf, Xl等后缀是强化/随机属性版本）
            if(search(item_name, "Xf") != -1) continue;
            if(search(item_name, "Xl") != -1) continue;
            if(search(item_name, "Xa") != -1) continue;
            if(search(item_name, "Xh") != -1) continue;
            if(search(item_name, ".pike") != -1) continue;
            if(search(item_name, ".") == 0 && item_name != ".") continue;

            string item_file = area_path + "/" + item_name;

            // 检查是否是普通文件（跳过目录）
            object stat = file_stat(item_file);
            if(!stat || stat->isdir) continue;

            // 读取物品文件内容
            string content = Stdio.read_file(item_file);
            if(!content) continue;

            mapping item_data = parse_item(content, area, item_name);
            if(item_data && item_data["name_cn"]) {
                string item_id = area + "/" + item_name;
                all_items[item_id] = item_data;
                item_count++;

                // 每100个物品输出一次进度
                if(item_count % 100 == 0) {
                    werror("已提取 %d 个物品...\n", item_count);
                }
            }
        }
    }

    werror("总共提取了 %d 个物品\n", item_count);

    // 输出为JSON
    string json = Standards.JSON.encode(all_items);
    Stdio.write_file(OUTPUT_FILE, json);
    werror("物品数据已保存到: %s\n", OUTPUT_FILE);
}

mapping parse_item(string content, string area, string item_name) {
    mapping item = ([]);
    array(string) lines = content / "\n";

    item["id"] = area + "/" + item_name;
    item["area"] = area;
    item["name"] = item_name;

    // 默认值
    item["level"] = 1;
    item["value"] = 0;
    item["weight"] = 100;

    // 检测物品类型
    if(search(content, "inherit WAPMUD_WEAPON") != -1) {
        item["item_type"] = "weapon";
        item["attack_power"] = 0;
        item["parry_power"] = 0;
    }
    else if(search(content, "inherit WAPMUD_ARMOR") != -1) {
        item["item_type"] = "armor";
        item["parry_power"] = 0;
        item["type"] = "armor";
    }
    else if(search(content, "inherit WAPMUD_FOOD") != -1) {
        item["item_type"] = "food";
        item["food_remaining"] = 1;
    }
    else {
        item["item_type"] = "misc";
    }

    foreach(lines, string line) {
        line = String.trim_all_whites(line);

        // 解析 name_cn
        string name_cn;
        if(sscanf(line, "name_cn=\"%s\"", name_cn) == 1 ||
           sscanf(line, "name_cn=\"%s\";", name_cn) == 1) {
            item["name_cn"] = name_cn;
        }

        // 解析 desc
        string desc;
        if(sscanf(line, "desc=\"%s\"", desc) == 1) {
            item["desc"] = desc;
        }

        // 解析 unit
        string unit;
        if(sscanf(line, "unit=\"%s\"", unit) == 1 ||
           sscanf(line, "unit=\"%s\";", unit) == 1) {
            item["unit"] = unit;
        }

        // 解析 skill (武器技能)
        string skill;
        if(sscanf(line, "skill=\"%s\"", skill) == 1 ||
           sscanf(line, "skill=\"%s\";", skill) == 1) {
            item["skill"] = skill;
        }

        // 解析 level
        int level;
        if(sscanf(line, "level=%d;", level) == 1) {
            item["level"] = level;
        }

        // 解析 value
        int value;
        if(sscanf(line, "value=%d;", value) == 1) {
            item["value"] = value;
        }

        // 解析 weight
        int weight;
        if(sscanf(line, "weight=%d;", weight) == 1) {
            item["weight"] = weight;
        }

        // 解析 attack_power (武器)
        int attack_power;
        if(sscanf(line, "attack_power=%d;", attack_power) == 1) {
            item["attack_power"] = attack_power;
        }

        // 解析 parry_power (武器/护甲)
        int parry_power;
        if(sscanf(line, "parry_power=%d;", parry_power) == 1) {
            item["parry_power"] = parry_power;
        }

        // 解析 armor type (护甲部位)
        string type;
        if(sscanf(line, "type=\"%s\"", type) == 1 ||
           sscanf(line, "type=\"%s\";", type) == 1) {
            item["type"] = type;
        }

        // 解析食物属性
        int food_remaining;
        if(sscanf(line, "food_remaining=%d;", food_remaining) == 1) {
            item["food_remaining"] = food_remaining;
        }

        int jing_supply;
        if(sscanf(line, "jing_supply=%d;", jing_supply) == 1) {
            item["jing_supply"] = jing_supply;
        }

        int shen_supply;
        if(sscanf(line, "shen_supply=%d;", shen_supply) == 1) {
            item["shen_supply"] = shen_supply;
        }

        int qi_supply;
        if(sscanf(line, "qi_supply=%d;", qi_supply) == 1) {
            item["qi_supply"] = qi_supply;
        }
    }

    return item;
}

int main(int argc, array(string) argv) {
    werror("开始提取物品数据...\n");
    extract_items();
    return 0;
}
