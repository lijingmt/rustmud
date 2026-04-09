#!/usr/bin/env pike
// 提取 txpike9 所有商店数据

#define ROOT "/usr/local/games/txpike9"
#define ROOM_DIR ROOT "/gamenv/d"
#define OUTPUT_FILE ROOT "/shops_data.json"

void extract_shops() {
    mapping all_shops = ([]);
    int shop_count = 0;

    // 获取所有区域目录
    array(string) areas = get_dir(ROOM_DIR);

    werror("发现 %d 个区域\n", sizeof(areas));

    foreach(areas, string area) {
        // 跳过非目录文件
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

            // 检查是否是商店
            if(search(content, "inherit WAPMUD_STORE") == -1) continue;

            mapping shop_data = parse_shop(content, area, room_name);
            if(shop_data && shop_data["name_cn"]) {
                string shop_id = area + "/" + room_name;
                all_shops[shop_id] = shop_data;
                shop_count++;

                if(shop_count % 10 == 0) {
                    werror("已提取 %d 个商店...\n", shop_count);
                }
            }
        }
    }

    werror("总共提取了 %d 个商店\n", shop_count);

    // 输出为JSON
    string json = Standards.JSON.encode(all_shops);
    Stdio.write_file(OUTPUT_FILE, json);
    werror("商店数据已保存到: %s\n", OUTPUT_FILE);
}

mapping parse_shop(string content, string area, string room_name) {
    mapping shop = ([]);
    array(string) lines = content / "\n";

    shop["id"] = area + "/" + room_name;
    shop["area"] = area;
    shop["name"] = room_name;

    array(mapping) goods = ({});

    foreach(lines, string line) {
        line = String.trim_all_whites(line);

        // 解析 name_cn
        string name_cn;
        if(sscanf(line, "name_cn=\"%s\"", name_cn) == 1 ||
           sscanf(line, "name_cn=\"%s\";", name_cn) == 1) {
            shop["name_cn"] = name_cn;
        }

        // 解析 desc
        string desc;
        if(sscanf(line, "desc=\"%s\"", desc) == 1) {
            shop["desc"] = desc;
        }

        // 解析 add_goods
        string short_name, full_path;
        if(sscanf(line, "add_goods(\"%s\",ROOT \"%s\");", short_name, full_path) == 2 ||
           sscanf(line, "add_goods(\"%s\",ROOT \"%s\")", short_name, full_path) == 2) {
            // 移除 "gamenv/clone/item/" 前缀，只保留类别/物品名
            string item_path = full_path;
            if(search(item_path, "gamenv/clone/item/") == 0) {
                item_path = item_path[18..]; // 跳过 "gamenv/clone/item/"
            }
            goods += ({([
                "short_name": short_name,
                "item_path": item_path,
            ])});
        }

        // 检测是否是当铺
        if(search(content, "int is_pawnshop()") != -1) {
            if(search(content, "return 1;") > search(content, "int is_pawnshop()")) {
                shop["is_pawnshop"] = 1;
            }
        }
    }

    if(sizeof(goods) > 0) {
        shop["goods"] = goods;
    }

    return shop;
}

int main(int argc, array(string) argv) {
    werror("开始提取商店数据...\n");
    extract_shops();
    return 0;
}
