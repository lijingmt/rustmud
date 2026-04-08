/**
 * Vue游戏客户端 - iframe模式 (显示原始HTML)
 */
const { createApp } = Vue;

// SHA-256 哈希函数（支持安全和非安全上下文）
async function sha256(message) {
    // 优先使用 SubtleCrypto API (更快的原生实现)
    if (window.crypto && window.crypto.subtle) {
        try {
            const encoder = new TextEncoder();
            const data = encoder.encode(message);
            const hashBuffer = await crypto.subtle.digest('SHA-256', data);
            const hashArray = Array.from(new Uint8Array(hashBuffer));
            return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
        } catch (e) {
            // Fall through to JS implementation
        }
    }

    // Fallback: 简单的 JS SHA-256 实现（用于非安全上下文如 HTTP）
    return sha256Fallback(message);
}

// SHA-256 fallback 纯 JS 实现
function sha256Fallback(str) {
    const K = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ae, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
    ];

    function rightRotate(n, x) {
        return ((x >>> n) | (x << (32 - n))) >>> 0;
    }

    // 转换字符串为字节数组
    const msgBuffer = new TextEncoder().encode(str);
    const msgLen = msgBuffer.length;

    // 计算填充后的长度: 必须是64字节的倍数
    // 1字节0x80 + (n字节0填充) + 8字节长度
    const lenInBits = msgLen * 8;
    // 找到需要填充的位置: (msgLen + 1 + 8) <= 64的倍数 - 8
    const paddingLen = (64 - ((msgLen + 1 + 8) % 64)) % 64;
    const totalLen = msgLen + 1 + paddingLen + 8;

    // 创建完整消息缓冲区
    const buffer = new Uint8Array(totalLen);
    buffer.set(msgBuffer, 0);
    buffer[msgLen] = 0x80;  // 添加1位后跟7个0

    // 添加64位大端序长度到最后8字节
    for (let i = 0; i < 8; i++) {
        buffer[totalLen - 8 + i] = (lenInBits >>> (56 - i * 8)) & 0xff;
    }

    // 初始化哈希值
    let h0 = 0x6a09e667, h1 = 0xbb67ae85, h2 = 0x3c6ef372, h3 = 0xa54ff53a;
    let h4 = 0x510e527f, h5 = 0x9b05688c, h6 = 0x1f83d9ab, h7 = 0x5be0cd19;

    // 处理消息 (64字节/16个32位字 为一块)
    const dataView = new DataView(buffer.buffer);

    for (let i = 0; i < totalLen; i += 64) {
        // 读取16个32位字(大端序)
        const w = new Uint32Array(64);
        for (let j = 0; j < 16; j++) {
            w[j] = dataView.getUint32(i + j * 4, false);
        }
        for (let j = 16; j < 64; j++) {
            const s0 = rightRotate(7, w[j - 15]) ^ rightRotate(18, w[j - 15]) ^ (w[j - 15] >>> 3);
            const s1 = rightRotate(17, w[j - 2]) ^ rightRotate(19, w[j - 2]) ^ (w[j - 2] >>> 10);
            w[j] = (w[j - 16] + s0 + w[j - 7] + s1) >>> 0;
        }

        let a = h0, b = h1, c = h2, d = h3, e = h4, f = h5, g = h6, h = h7;
        for (let j = 0; j < 64; j++) {
            const S1 = rightRotate(6, e) ^ rightRotate(11, e) ^ rightRotate(25, e);
            const ch = (e & f) ^ (~e & g);
            const temp1 = (h + S1 + ch + K[j] + w[j]) >>> 0;
            const S0 = rightRotate(2, a) ^ rightRotate(13, a) ^ rightRotate(22, a);
            const maj = (a & b) ^ (a & c) ^ (b & c);
            const temp2 = (S0 + maj) >>> 0;
            h = g; g = f; f = e; e = (d + temp1) >>> 0;
            d = c; c = b; b = a; a = (temp1 + temp2) >>> 0;
        }
        h0 = (h0 + a) >>> 0; h1 = (h1 + b) >>> 0; h2 = (h2 + c) >>> 0; h3 = (h3 + d) >>> 0;
        h4 = (h4 + e) >>> 0; h5 = (h5 + f) >>> 0; h6 = (h6 + g) >>> 0; h7 = (h7 + h) >>> 0;
    }

    // 转换为十六进制
    const hex = (n) => n.toString(16).padStart(8, '0');
    return hex(h0) + hex(h1) + hex(h2) + hex(h3) + hex(h4) + hex(h5) + hex(h6) + hex(h7);
}

createApp({
    data() {
        return {
            showLogin: true,
            headerMenuOpen: false,
            showRegister: false,
            isLoggingIn: false,
            isRegistering: false,
            loginError: '',
            registerError: '',
            registerSuccess: false,
            loginForm: {
                partition: 'tx01',
                userid: '',
                password: ''
            },
            registerForm: {
                partition: 'tx01',
                userid: '',
                password: '',
                passwordConfirm: '',
                captcha: ''
            },
            captchaCode: '',
            partitions: [],  // 将从API动态加载
            partitionsLoading: true,
            txd: '',
            apiBase: '',
            gameFrameUrl: '',
            frameLoading: true,
            showCommandInput: false,
            commandInput: '',
            showChatInput: false,
            chatInput: '',
            showChatRoom: false,  // 是否显示聊天室视图
            chatMessages: [],  // 聊天消息列表
            chatChannel: 'pub_channel',  // 当前聊天频道
            chatPollingInterval: null,  // 聊天轮询定时器
            theme: 'classic',  // classic or dark，默认经典模式
            playerStats: null,  // 玩家状态信息
            statsInterval: null,  // 状态更新定时器
            autofightInterval: null,  // 自动战斗定时器
            lastCommand: 'look',  // 上一次执行的命令，默认是look
            pendingRequests: {},  // 正在进行的异步请求
            useAsyncMode: false,  // 是否使用异步模式（同步更快，无轮询开销）
            // JSON模式 (vue-ui-3: 无iframe，Vue直接渲染)
            useJsonMode: true,  // 使用JSON模式代替iframe
            mudLines: [],  // MUD输出行数组
            mudLoading: false,  // MUD加载中状态
            // 战斗系统
            isInBattle: false,  // 是否处于战斗状态
            battleMiniMode: true,  // 迷你模式：只显示HP条
            battleFullscreen: false,  // 全屏模式：遮住整个页面
            battleShowLog: false,  // 显示战斗日志
            battleLog: [],  // 战斗日志条目
            battleAnimations: [],  // 当前显示的战斗动画
            battleEnemy: null,  // 当前敌人信息 {name, hp, hpMax}
            battleEnemyFull: null,  // 敌人完整状态（从API获取）
            battlePlayerFull: null,  // 玩家完整状态（从API获取）
            battleStatusInterval: null,  // 战斗状态轮询定时器
            skillAnimations: [],  // 武侠技能动画列表
            // 招式系统
            showPerformsList: false,  // 显示招式列表
            performsData: null,  // 招式数据
            performsLoading: false,  // 招式加载中
            // 快捷菜单
            quickActionsCollapsed: false  // 快捷菜单是否折叠
        };
    },

    watch: {
        // 监听 mudLines 变化，更新后重新翻译并滚动到顶部
        mudLines() {
            this.$nextTick(() => {
                this.reapplyTranslation();
                // 每次更新后滚动到顶部
                const container = document.querySelector('.mud-output-container');
                if (container) {
                    container.scrollTop = 0;
                    // 根据行数动态调整高度
                    this.adjustContainerHeight();
                }
            });
        }
    },

    methods: {
        // 重新应用翻译（用于 mudLines 更新后）
        reapplyTranslation() {
            const savedLang = localStorage.getItem('userLanguage');
            if (savedLang && savedLang !== 'chinese_simplified' && typeof translate !== 'undefined') {
                // 用户选择了非简体中文语言，重新翻译新内容
                translate.execute();
            }
        },

        detectApiBase() {
            const hostname = window.location.hostname;
            const protocol = window.location.protocol;
            // 从环境变量获取API端口，构建时会被替换为实际端口
            const apiPort = '8888';

            // localhost 始终使用配置的端口
            if (hostname === 'localhost' || hostname === '127.0.0.1') {
                return protocol + '//localhost:' + apiPort;
            }

            // HTTPS 时使用不带端口的地址（由反向代理转发）
            // HTTP 时使用配置的端口
            if (protocol === 'https:') {
                return protocol + '//' + hostname;
            }
            return protocol + '//' + hostname + ':' + apiPort;
        },

        // 生成验证码
        refreshCaptcha() {
            const chars = '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';
            let code = '';
            for (let i = 0; i < 4; i++) {
                code += chars.charAt(Math.floor(Math.random() * chars.length));
            }
            this.captchaCode = code;
        },

        // 打开注册页面
        openRegister() {
            this.showLogin = false;
            this.showRegister = true;
            this.registerError = '';
            this.registerSuccess = false;
        },

        // 关闭注册页面
        closeRegister() {
            this.showRegister = false;
            this.showLogin = true;
        },

        // 从API加载分区列表
        async loadPartitions() {
            try {
                const response = await fetch(`${this.apiBase}/api/partitions`);
                if (!response.ok) {
                    console.error('加载分区列表失败:', response.status);
                    // 使用默认分区列表
                    this.partitions = this.getDefaultPartitions();
                    return;
                }
                const data = await response.json();
                this.partitions = data.partitions || [];
                if (this.partitions.length > 0) {
                    this.loginForm.partition = this.partitions[0].value;
                    this.registerForm.partition = this.partitions[0].value;
                }
                console.log('已加载分区列表:', this.partitions);
            } catch (e) {
                console.error('加载分区列表异常:', e);
                // 使用默认分区列表
                this.partitions = this.getDefaultPartitions();
            } finally {
                this.partitionsLoading = false;
            }
        },

        // 默认分区列表（API失败时使用）
        getDefaultPartitions() {
            return [
                { value: 'tx01', label: '原1区' },
                { value: 'tx02', label: '原2区' },
                { value: 'tx03', label: '原3区' },
                { value: 'tx04', label: '原4区' },
                { value: 'tx05', label: '原5区' },
                { value: 'tx06', label: '原6区' }
            ];
        },

        // 生成游戏iframe URL
        getGameFrameUrl() {
            if (!this.txd) return '';
            return `${this.apiBase}/api/html?txd=${encodeURIComponent(this.txd)}&cmd=look`;
        },

        // 注册功能
        async doRegister() {
            // 验证输入
            if (!this.registerForm.userid || !this.registerForm.password) {
                this.registerError = '账号和密码不能为空';
                return;
            }
            if (this.registerForm.userid.length < 2 || this.registerForm.password.length < 2) {
                this.registerError = '账号和密码不能少于2个字符';
                return;
            }
            if (this.registerForm.userid.length > 12 || this.registerForm.password.length > 12) {
                this.registerError = '账号和密码不能超过12个字符';
                return;
            }
            if (!/^[a-zA-Z0-9]+$/.test(this.registerForm.userid + this.registerForm.password)) {
                this.registerError = '账号和密码只能是大小写字母或数字';
                return;
            }
            if (this.registerForm.password !== this.registerForm.passwordConfirm) {
                this.registerError = '两次输入的密码不一致';
                return;
            }
            if (this.registerForm.captcha.toLowerCase() !== this.captchaCode.toLowerCase()) {
                this.registerError = '验证码错误';
                this.refreshCaptcha();
                return;
            }

            this.isRegistering = true;
            this.registerError = '';
            this.registerSuccess = false;

            try {
                const fullUserid = this.registerForm.partition + this.registerForm.userid;
                // 生成一个随机的session ID作为验证码
                const sessionId = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);

                // 获取challenge用于密码哈希
                const challengeResp = await fetch(this.apiBase + '/api/challenge');
                if (!challengeResp.ok) {
                    this.registerError = '获取安全挑战失败';
                    return;
                }
                const challengeData = await challengeResp.json();
                const challenge = challengeData.challenge;

                // 使用challenge对密码进行哈希
                const passwordHash = await sha256(challenge + this.registerForm.password);

                // 发送注册命令: login_regnew gamenv fullUserid passwordHash challenge sessionId
                // 注意：注册不需要txd，直接发送cmd参数
                const cmd = `login_regnew gamenv ${fullUserid} ${passwordHash} ${sessionId} ${challenge}`;
                const url = this.apiBase + '/api/html?cmd=' + encodeURIComponent(cmd);

                console.log('=== 注册请求开始 ===');
                console.log('apiBase:', this.apiBase);
                console.log('cmd:', cmd);
                console.log('url:', url);

                const response = await fetch(url, {
                    method: 'GET'
                });

                console.log('response.status:', response.status);
                console.log('response.ok:', response.ok);

                const text = await response.text();
                console.log('注册响应:', text);

                // 检查注册结果
                if (text.includes('error1') || text.includes('已经有人使用')) {
                    this.registerError = '该账号已存在，请修改后重试';
                } else if (text.includes('error2') || text.includes('登录错误')) {
                    this.registerError = '注册失败，请稍后重试';
                } else {
                    // 注册成功 - 响应格式: username,password
                    this.registerSuccess = true;
                    this.registerError = '';
                    // 延迟后返回登录页面
                    setTimeout(() => {
                        this.showRegister = false;
                        this.showLogin = true;
                        this.registerSuccess = false;
                        // 填充登录表单
                        this.loginForm.partition = this.registerForm.partition;
                        this.loginForm.userid = this.registerForm.userid;
                        this.loginForm.password = this.registerForm.password;
                    }, 2000);
                }
            } catch (e) {
                console.error('注册请求失败:', e);
                console.error('错误名称:', e.name);
                console.error('错误消息:', e.message);
                this.registerError = '连接失败: ' + e.message;
            } finally {
                this.isRegistering = false;
            }
        },

        async doLogin() {
            if (!this.loginForm.userid || !this.loginForm.password) {
                this.loginError = '请输入账号和密码';
                return;
            }
            this.isLoggingIn = true;
            this.loginError = '';
            try {
                const fullUserid = this.loginForm.partition + this.loginForm.userid;

                // 获取challenge用于密码哈希
                const challengeResp = await fetch(this.apiBase + '/api/challenge');
                if (!challengeResp.ok) {
                    this.loginError = '获取安全挑战失败';
                    return;
                }
                const challengeData = await challengeResp.json();
                const challenge = challengeData.challenge;

                // 使用challenge对密码进行哈希
                const passwordHash = await sha256(challenge + this.loginForm.password);

                if (this.useJsonMode) {
                    // JSON模式: 使用 /api/json 接口登录
                    const params = new URLSearchParams({
                        userid: fullUserid,
                        password: passwordHash,
                        challenge: challenge,
                        cmd: 'init'
                    });

                    const response = await fetch(this.apiBase + '/api/json?' + params.toString(), {
                        method: 'GET'
                    });

                    if (!response.ok) {
                        if (response.status === 401) {
                            this.loginError = '用户名或密码错误';
                        } else {
                            this.loginError = '登录失败: HTTP ' + response.status;
                        }
                        return;
                    }

                    const data = await response.json();
                    if (data.error) {
                        this.loginError = data.error || '登录失败';
                        return;
                    }

                    // 登录成功，生成 txd 用于后续请求
                    this.txd = this.encodeTxd(fullUserid, this.loginForm.password);
                    // 如果响应中有新的txd，使用它
                    if (data.txd) {
                        this.txd = data.txd;
                    }
                    localStorage.setItem('mud_txd', this.txd);
                    localStorage.setItem('mud_partition', this.loginForm.partition);
                    localStorage.setItem('mud_userid', this.loginForm.userid);

                    // 更新MUD输出
                    this.mudLines = data.lines || [];

                    // 隐藏登录界面
                    this.showLogin = false;

                    // 开始更新玩家状态
                    this.startStatsUpdate();
                } else {
                    // iframe模式: 使用 /api/html 接口
                    const params = new URLSearchParams({
                        userid: fullUserid,
                        password: passwordHash,
                        challenge: challenge,
                        cmd: 'look'
                    });

                    const response = await fetch(this.apiBase + '/api/html?' + params.toString(), {
                        method: 'GET'
                    });

                    if (!response.ok) {
                        if (response.status === 401) {
                            this.loginError = '用户名或密码错误';
                        } else {
                            this.loginError = '登录失败: HTTP ' + response.status;
                        }
                        return;
                    }

                    // 检查响应是否包含错误
                    const text = await response.text();
                    if (text.includes('登录错误') || text.includes('用户名不存在')) {
                        this.loginError = '用户名或密码错误';
                        return;
                    }

                    // 登录成功，生成 txd 用于后续请求
                    this.txd = this.encodeTxd(fullUserid, this.loginForm.password);
                    localStorage.setItem('mud_txd', this.txd);
                    localStorage.setItem('mud_partition', this.loginForm.partition);
                    localStorage.setItem('mud_userid', this.loginForm.userid);

                    // 设置iframe URL
                    this.gameFrameUrl = this.getGameFrameUrl();
                    this.showLogin = false;

                    // 开始更新玩家状态
                    this.startStatsUpdate();
                }
            } catch (e) {
                this.loginError = '连接失败: ' + e.message;
            } finally {
                this.isLoggingIn = false;
            }
        },

        encodeTxd(userid, password) {
            let uid = '';
            let pid = '';
            for (let i = 0; i < userid.length; i++) {
                let code = userid.charCodeAt(i);
                if (Math.floor(i / 2) === 0) {
                    uid += (code === 121) ? '%7B' : String.fromCharCode(code + 2);
                } else {
                    uid += (code === 122) ? '%7B' : String.fromCharCode(code + 1);
                }
            }
            for (let i = 0; i < password.length; i++) {
                let code = password.charCodeAt(i);
                if (Math.floor(i / 2) === 0) {
                    pid += (code === 122) ? '%7B' : String.fromCharCode(code + 1);
                } else {
                    if (code === 121) {
                        pid += '%7B';
                    } else if (code === 122) {
                        pid += '%7C';
                    } else {
                        pid += String.fromCharCode(code + 2);
                    }
                }
            }
            return uid + '~' + pid;
        },

        // iframe加载完成
        onFrameLoad() {
            this.frameLoading = false;
        },

        // 刷新iframe
        refreshFrame() {
            if (this.useJsonMode) {
                // JSON模式: 重新执行最后命令或look
                this.sendJsonCommand(this.lastCommand || 'look');
            } else {
                // iframe模式: 强制刷新iframe
                this.frameLoading = true;
                const iframe = this.$refs.gameFrame;
                if (iframe) {
                    iframe.src = iframe.src;
                }
            }
        },

        // 显示命令输入框
        showCommandModal() {
            this.showCommandInput = true;
            this.$nextTick(() => {
                if (this.$refs.commandInputRef) {
                    this.$refs.commandInputRef.focus();
                }
            });
        },

        // 显示聊天输入框 - 改为打开聊天室视图
        showChatModal() {
            this.showChatRoom = true;
            // 先执行游戏内的打开聊天命令，设置roomchatid
            this.sendQuickCommand('ui_select_room open');
            // 延迟启动轮询，等待命令执行
            setTimeout(() => {
                this.startChatPolling();
                this.loadChatMessages();
            }, 300);
            // 聚焦输入框
            this.$nextTick(() => {
                if (this.$refs.chatInputRef) {
                    this.$refs.chatInputRef.focus();
                }
            });
        },

        // 关闭聊天室
        closeChatRoom() {
            this.showChatRoom = false;
            this.stopChatPolling();
        },

        // 开始轮询聊天消息
        startChatPolling() {
            // 清除已有定时器
            this.stopChatPolling();
            // 每2秒轮询一次
            this.chatPollingInterval = setInterval(() => {
                this.loadChatMessages();
            }, 2000);
        },

        // 停止轮询聊天消息
        stopChatPolling() {
            if (this.chatPollingInterval) {
                clearInterval(this.chatPollingInterval);
                this.chatPollingInterval = null;
            }
        },

        // 加载聊天消息
        async loadChatMessages() {
            // 只在聊天室打开时才加载
            if (!this.txd || !this.showChatRoom) return;

            try {
                const url = `${this.apiBase}/api/chat/messages?txd=${encodeURIComponent(this.txd)}&channel=${encodeURIComponent(this.chatChannel)}`;
                const response = await fetch(url);
                if (response.ok) {
                    const data = await response.json();
                    if (data.messages) {
                        // 更新消息列表
                        this.chatMessages = data.messages;
                        // 滚动到底部
                        this.$nextTick(() => {
                            this.scrollChatToBottom();
                        });
                    }
                }
            } catch (e) {
                console.error('加载聊天消息失败:', e);
            }
        },

        // 滚动聊天到底部
        scrollChatToBottom() {
            const container = this.$refs.chatMessagesContainer;
            if (container) {
                container.scrollTop = container.scrollHeight;
            }
        },

        // 发送聊天消息
        async sendChat() {
            const msg = this.chatInput.trim();
            if (!msg) return;

            // 通过API发送消息
            try {
                const response = await fetch(`${this.apiBase}/api/chat/send`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                    body: new URLSearchParams({
                        txd: this.txd,
                        channel: this.chatChannel,
                        message: msg
                    })
                });

                if (response.ok) {
                    this.chatInput = '';
                    // 立即刷新消息
                    this.loadChatMessages();
                } else {
                    console.error('发送消息失败');
                }
            } catch (e) {
                console.error('发送消息失败:', e);
            }
        },

        // 发送命令
        sendCommand() {
            const cmd = this.commandInput.trim();
            if (!cmd) return;

            // JSON模式优先
            if (this.useJsonMode) {
                this.lastCommand = cmd;
                this.sendJsonCommand(cmd);
                this.commandInput = '';
                this.showCommandInput = false;
            } else if (this.useAsyncMode) {
                this.sendCommandAsync();
            } else {
                // 同步模式（原有方式）
                const url = `${this.apiBase}/api/html?txd=${encodeURIComponent(this.txd)}&cmd=${encodeURIComponent(cmd)}`;
                const iframe = this.$refs.gameFrame;
                if (iframe) {
                    this.frameLoading = true;
                    iframe.src = url;
                }

                this.commandInput = '';
                this.showCommandInput = false;
            }
        },

        // 发送聊天消息 - 使用ui_chat命令
        sendChat() {
            const msg = this.chatInput.trim();
            if (!msg) return;

            // 使用 ui_chat 命令发送消息
            const cmd = 'ui_chat ' + msg;

            if (this.useJsonMode) {
                // JSON模式: 直接调用 sendJsonCommand
                this.sendJsonCommand(cmd);
            } else {
                // iframe模式: 更新iframe src
                const url = `${this.apiBase}/api/html?txd=${encodeURIComponent(this.txd)}&cmd=${encodeURIComponent(cmd)}`;
                const iframe = this.$refs.gameFrame;
                if (iframe) {
                    this.frameLoading = true;
                    iframe.src = url;
                }
            }

            this.chatInput = '';
            this.showChatInput = false;
        },

        // 快捷命令
        sendQuickCommand(cmd) {
            // 点击快捷按钮时立即滚动到顶部
            window.scrollTo({ top: 0, behavior: 'smooth' });
            const mudContainer = document.querySelector('.mud-output-container');
            if (mudContainer) {
                mudContainer.scrollTop = 0;
            }

            // 记录最后执行的命令
            this.lastCommand = cmd;

            // JSON模式 (vue-ui-3): 直接渲染，无iframe
            if (this.useJsonMode) {
                this.sendJsonCommand(cmd);
            } else if (this.useAsyncMode) {
                // 异步iframe模式
                this.sendQuickCommandAsync(cmd);
            } else {
                // 同步iframe模式（原有方式）
                const url = `${this.apiBase}/api/html?txd=${encodeURIComponent(this.txd)}&cmd=${encodeURIComponent(cmd)}`;
                const iframe = this.$refs.gameFrame;
                if (iframe) {
                    this.frameLoading = true;
                    iframe.src = url;
                }
            }
        },

        // JSON模式: 发送命令并获取结构化数据
        async sendJsonCommand(cmd, isRetry = false) {
            // 滚动到顶部（同时滚动window和MUD容器）
            window.scrollTo({ top: 0, behavior: 'smooth' });
            const mudContainer = document.querySelector('.mud-output-container');
            if (mudContainer) {
                mudContainer.scrollTop = 0;
            }

            console.log('[sendJsonCommand] 发送命令:', cmd);
            console.log('[sendJsonCommand] apiBase:', this.apiBase);
            console.log('[sendJsonCommand] txd:', this.txd);

            this.mudLoading = true;
            try {
                const url = `${this.apiBase}/api/json?txd=${encodeURIComponent(this.txd)}&cmd=${encodeURIComponent(cmd)}`;
                console.log('[sendJsonCommand] 完整URL:', url);

                const response = await fetch(url);
                console.log('[sendJsonCommand] 响应状态:', response.status);

                if (!response.ok) {
                    // 401 表示未授权（会话已过期），尝试重新登录并重试命令
                    if (response.status === 401 && !isRetry) {
                        console.log('[会话过期] 尝试重新登录并重试命令...');
                        await this.relogin();
                        // 重新登录成功后重试原始命令
                        if (!this.showLogin) {
                            console.log('[重试命令]', cmd);
                            return this.sendJsonCommand(cmd, true);
                        }
                    }
                    throw new Error(`HTTP ${response.status}`);
                }
                const data = await response.json();
                console.log('[sendJsonCommand] 响应数据:', data);

                if (data.error) {
                    console.error('命令执行错误:', data.error);
                    // 如果是认证错误，尝试重新登录并重试命令
                    if ((data.error.includes('认证') || data.error.includes('登录') || data.error.includes('未登录')) && !isRetry) {
                        console.log('[会话过期] 尝试重新登录并重试命令...');
                        await this.relogin();
                        // 重新登录成功后重试原始命令
                        if (!this.showLogin) {
                            console.log('[重试命令]', cmd);
                            return this.sendJsonCommand(cmd, true);
                        }
                    }
                    return;
                }
                // 更新txd（可能已变化）
                if (data.txd) {
                    this.txd = data.txd;
                    localStorage.setItem('mud_txd', this.txd);
                }
                // 更新MUD输出
                this.mudLines = data.lines || [];
                console.log('[sendJsonCommand] mudLines数量:', this.mudLines.length);

                // 检测战斗状态
                this.checkBattleStatus();
                // 解析战斗动作并生成动画
                this.parseBattleActions(data.lines || []);
            } catch (e) {
                console.error('JSON命令执行失败:', e);
                // 网络错误也尝试重新登录并重试
                if ((e.message.includes('401') || e.message.includes('Unauthorized')) && !isRetry) {
                    console.log('[会话过期] 尝试重新登录并重试命令...');
                    await this.relogin();
                    // 重新登录成功后重试原始命令
                    if (!this.showLogin) {
                        console.log('[重试命令]', cmd);
                        return this.sendJsonCommand(cmd, true);
                    }
                }
            } finally {
                this.mudLoading = false;
            }
        },

        // JSON模式: 获取按钮样式类名
        getButtonClass(label) {
            if (label.includes('东→') || label.includes('西←') ||
                label.includes('南↓') || label.includes('北↑')) {
                return 'btn btn-outline-success btn-sm';
            } else if (label.includes('杀戮') || label.includes('商城') ||
                       label.includes('锻造')) {
                return 'btn btn-outline-warning btn-sm';
            } else if (label.includes('吃药')) {
                return 'btn btn-outline-purple btn-sm';
            }
            return 'btn btn-outline-info btn-sm';
        },

        // JSON模式: 获取颜色样式类名
        getColorClass(colorCode) {
            const colorMap = {
                0x30: 'color-black',
                0x31: 'color-red-bold',
                0x32: 'color-green-bold',
                0x33: 'color-blue-bold',
                0x34: 'color-cyan-bold',
                0x35: 'color-purple-bold',
                0x36: 'color-orange-bold',
                0x37: 'color-gray',
                0x38: 'color-dark-gray',
                0x39: 'color-light-gray',
                0x67: 'color-gold'
            };
            return colorMap[colorCode] || '';
        },

        // JSON模式: 渲染文本部分（处理颜色和图片）
        renderTextParts(parts) {
            if (!parts) return '';
            let html = '';
            let inSpan = false;

            for (const part of parts) {
                if (part.type === 'color-start') {
                    if (inSpan) html += '</span>';
                    html += `<span class="${part.class}">`;
                    inSpan = true;
                } else if (part.type === 'color-end') {
                    if (inSpan) {
                        html += '</span>';
                        inSpan = false;
                    }
                } else if (part.type === 'text') {
                    // 解析 [imgurl picture:...] 格式为图片
                    html += this.parseInlineImages(part.content);
                }
            }
            if (inSpan) html += '</span>';
            return html;
        },

        // 解析文本中的内联图片 [imgurl picture:/images/...]
        parseInlineImages(text) {
            if (!text) return '';
            // 匹配 [imgurl picture:路径] 格式
            return text.replace(/\[imgurl\s+picture:([^\]]+)\]/g, (match, imagePath) => {
                // 构建完整图片URL
                const imageUrl = this.getImageUrl(imagePath);
                return `<img src="${imageUrl}" class="mud-inline-image" alt="图片" onerror="this.style.display='none'">`;
            });
        },

        // 获取图片的完整URL
        getImageUrl(imagePath) {
            // imagePath 格式: /images/user/0_100.gif
            // 使用与当前页面相同的协议和主机名
            const protocol = window.location.protocol;
            const hostname = window.location.hostname;
            // 从环境变量获取API端口，构建时会被替换为实际端口
            const apiPort = '8888';

            // HTTPS 时使用主域名，HTTP 时使用配置的端口
            let baseUrl;
            if (protocol === 'https:') {
                baseUrl = protocol + '//' + hostname;
            } else {
                // 内网访问，使用配置的端口
                baseUrl = protocol + '//' + hostname + ':' + apiPort;
            }
            return baseUrl + imagePath;
        },

        // 解析聊天消息中的链接 [label:command argument]
        parseChatLinks(text) {
            if (!text) return '';
            // 匹配 [label:command argument] 格式
            // command 后面可能有空格和参数
            return text.replace(/\[([^\]]+?):([^\]]+?)\s+([^\]]+)\]/g, (match, label, command, arg) => {
                // 创建可点击的链接按钮
                return `<span class="chat-link" data-command="${command} ${arg}" onclick="handleChatLinkClick(this)">${label}</span>`;
            });
        },

        // JSON模式: 提交输入框
        submitInput(name, event) {
            console.log('[submitInput] 被调用, name:', name);
            console.log('[submitInput] event.target:', event.target);

            let inputValue = '';
            // 如果是输入框的回车事件
            if (event.target && event.target.tagName === 'INPUT') {
                inputValue = event.target.value || '';
                console.log('[submitInput] 从INPUT获取值:', inputValue);
            } else {
                // 如果是确定按钮的点击事件，通过ref获取输入框的值
                const refName = 'input-' + name;
                const inputRef = this.$refs[refName];
                console.log('[submitInput] refName:', refName, 'inputRef:', inputRef);
                if (inputRef && inputRef.length) {
                    inputValue = inputRef[0].value || '';
                    console.log('[submitInput] 从inputRef[0]获取值:', inputValue);
                } else if (inputRef) {
                    inputValue = inputRef.value || '';
                    console.log('[submitInput] 从inputRef获取值:', inputValue);
                }
            }
            const cmd = `${name} ${inputValue}`;
            console.log('[submitInput] 最终命令:', cmd);
            this.sendJsonCommand(cmd);
        },

        // JSON模式: 提交命令输入框
        submitCmdInput(cmdName, event) {
            console.log('[submitCmdInput] 被调用, cmdName:', cmdName);
            console.log('[submitCmdInput] event.target:', event.target);

            let inputValue = '';
            if (event.target && event.target.tagName === 'INPUT') {
                inputValue = event.target.value || '';
                console.log('[submitCmdInput] 从INPUT获取值:', inputValue);
            } else {
                // 找到同组的输入框
                const input = event.target.parentElement.querySelector('input');
                inputValue = input ? input.value : '';
                console.log('[submitCmdInput] 从兄弟input获取值:', inputValue);
            }
            const cmd = `${cmdName} ${inputValue}`;
            console.log('[submitCmdInput] 最终命令:', cmd);
            this.sendJsonCommand(cmd);
        },

        // 退出登录
        doLogout() {
            localStorage.removeItem('mud_txd');
            localStorage.removeItem('mud_partition');
            localStorage.removeItem('mud_userid');
            this.txd = '';
            this.gameFrameUrl = '';
            this.playerStats = null;
            this.stopStatsUpdate();
            // 清理自动战斗定时器
            if (this.autofightInterval) {
                clearInterval(this.autofightInterval);
                this.autofightInterval = null;
            }
            // 清理聊天轮询定时器
            this.stopChatPolling();
            this.showLogin = true;
        },

        // 自动重新登录（当会话过期时）
        async relogin() {
            const savedPartition = localStorage.getItem('mud_partition') || 'tx01';
            const savedUser = localStorage.getItem('mud_userid');
            const savedTxd = localStorage.getItem('mud_txd');
            if (!savedTxd || !savedUser) {
                // 没有保存的登录信息，显示登录界面
                this.showLogin = true;
                return;
            }

            try {
                // 解码密码从 txd
                const password = this.decodePasswordFromTxd(savedTxd);
                if (!password) {
                    throw new Error('无法解码密码');
                }

                const fullUserid = savedPartition + savedUser;

                // 获取 challenge
                const challengeResp = await fetch(this.apiBase + '/api/challenge');
                if (!challengeResp.ok) {
                    throw new Error('获取安全挑战失败');
                }
                const challengeData = await challengeResp.json();
                const challenge = challengeData.challenge;

                // 使用 challenge 对密码进行哈希
                const passwordHash = await sha256(challenge + password);

                // 发送登录请求
                const params = new URLSearchParams({
                    userid: fullUserid,
                    password: passwordHash,
                    challenge: challenge,
                    cmd: 'init'
                });

                const response = await fetch(this.apiBase + '/api/json?' + params.toString());

                if (!response.ok) {
                    throw new Error('登录失败: HTTP ' + response.status);
                }

                const data = await response.json();
                if (data.error) {
                    throw new Error(data.error);
                }

                // 重新登录成功
                this.txd = data.txd || this.encodeTxd(fullUserid, password);
                localStorage.setItem('mud_txd', this.txd);
                localStorage.setItem('mud_partition', savedPartition);
                localStorage.setItem('mud_userid', savedUser);

                // 更新 MUD 输出
                this.mudLines = data.lines || [];
                this.showLogin = false;

                console.log('[重新登录] 成功');
            } catch (e) {
                console.error('[重新登录] 失败:', e);
                // 重新登录失败，显示登录界面
                this.showLogin = true;
                this.loginForm.partition = savedPartition;
                this.loginForm.userid = savedUser;
            }
        },

        // 从 txd 解码密码（encodeTxd 的逆操作）
        decodePasswordFromTxd(txd) {
            try {
                const parts = txd.split('~');
                if (parts.length !== 2) return null;

                const pid = parts[1];  // 编码后的密码部分
                let password = '';

                for (let i = 0; i < pid.length; i++) {
                    const code = pid.charCodeAt(i);

                    // 处理 URL 编码的特殊字符
                    if (pid[i] === '%' && i + 2 < pid.length) {
                        const hex = pid.substring(i + 1, i + 3);
                        if (hex === '7B') {
                            if (Math.floor(i / 2) === 0) {
                                password += 'z';
                            } else {
                                password += 'y';
                            }
                            i += 2;
                            continue;
                        } else if (hex === '7C') {
                            password += 'z';
                            i += 2;
                            continue;
                        }
                    }

                    // 逆操作还原密码
                    if (Math.floor(i / 2) === 0) {
                        password += String.fromCharCode(code - 1);
                    } else {
                        password += String.fromCharCode(code - 2);
                    }
                }
                return password;
            } catch (e) {
                console.error('解码密码失败:', e);
                return null;
            }
        },

        // 返回界面选择
        goToSelection() {
            if (confirm('返回界面选择？')) {
                localStorage.removeItem('mud_txd');
                localStorage.removeItem('mud_partition');
                localStorage.removeItem('mud_userid');
                localStorage.removeItem('mud_ui_choice');
                localStorage.removeItem('mud_ui_choice_time');
                window.location.href = '../pc.jsp?ui=back';
            }
        },

        // 切换头部菜单
        toggleHeaderMenu() {
            this.headerMenuOpen = !this.headerMenuOpen;
        },

        // 切换主题
        toggleTheme() {
            // 三种主题循环：classic → dark → light → classic
            if (this.theme === 'classic') {
                this.theme = 'dark';
            } else if (this.theme === 'dark') {
                this.theme = 'light';
            } else {
                this.theme = 'classic';
            }
            localStorage.setItem('mud_theme', this.theme);
            this.applyTheme();
            // 刷新iframe以应用新主题
            if (this.txd && this.gameFrameUrl) {
                this.refreshFrame();
            }
        },

        // 应用主题到body
        applyTheme() {
            document.body.setAttribute('data-theme', this.theme);
        },

        // 获取玩家状态
        async fetchPlayerStats() {
            if (!this.txd) return;

            try {
                const response = await fetch(`${this.apiBase}/api/status?txd=${encodeURIComponent(this.txd)}`);
                if (response.ok) {
                    const data = await response.json();
                    if (!data.error) {
                        // 记录之前的 autofight 状态
                        const wasAutofight = this.playerStats && this.playerStats.autofight;
                        this.playerStats = data;
                        // 检查自动战斗状态变化
                        const isAutofight = this.playerStats && this.playerStats.autofight;
                        if (wasAutofight && !isAutofight) {
                            // 明确从开启变成关闭，清除定时器
                            if (this.autofightInterval) {
                                clearInterval(this.autofightInterval);
                                this.autofightInterval = null;
                            }
                        } else if (!wasAutofight && isAutofight) {
                            // 明确从关闭变成开启，启动定时器
                            this.checkAutofight();
                        }
                        // 如果状态没变，保持当前定时器状态不变
                    }
                }
            } catch (e) {
                console.error('获取玩家状态失败:', e);
                // 网络错误时，不清除 autofight 定时器，保持当前状态
            }
        },

        // 开始定时更新玩家状态
        startStatsUpdate() {
            this.fetchPlayerStats();
            // 每2秒更新一次（后端已用Thread异步，不会阻塞）
            this.statsInterval = setInterval(() => {
                this.fetchPlayerStats();
            }, 2000);
        },

        // 停止定时更新
        stopStatsUpdate() {
            if (this.statsInterval) {
                clearInterval(this.statsInterval);
                this.statsInterval = null;
            }
        },

        // 切换自动战斗
        async toggleAutofight() {
            if (!this.txd) return;

            try {
                const response = await fetch(`${this.apiBase}/api/autofight`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded',
                    },
                    body: new URLSearchParams({
                        txd: this.txd,
                        action: 'toggle'
                    })
                });

                if (response.ok) {
                    const data = await response.json();
                    // 刷新状态
                    await this.fetchPlayerStats();
                    // 显示提示
                    alert(data.message || '自动战斗状态已切换');
                } else {
                    alert('切换自动战斗失败');
                }
            } catch (e) {
                console.error('切换自动战斗失败:', e);
                alert('切换自动战斗失败: ' + e.message);
            }
        },

        // 检查并启动/停止自动战斗
        checkAutofight() {
            if (this.playerStats && this.playerStats.autofight) {
                // 开启自动战斗 - 每秒执行 flushview 命令
                if (!this.autofightInterval) {
                    this.autofightInterval = setInterval(() => {
                        if (this.txd) {
                            // 自动战斗使用 flushview 命令，而不是重复最后命令
                            // flushview 会自动处理：打怪、吃药、拾取、移动等
                            if (this.useJsonMode) {
                                // JSON模式: 直接调用 sendJsonCommand
                                this.sendJsonCommand('flushview');
                            } else {
                                // iframe模式: 更新iframe src
                                const url = `${this.apiBase}/api/html?txd=${encodeURIComponent(this.txd)}&cmd=${encodeURIComponent('flushview')}`;
                                const iframe = this.$refs.gameFrame;
                                if (iframe) {
                                    iframe.src = url;
                                }
                            }
                        }
                    }, 1000);  // 1秒执行一次
                }
            } else {
                // 关闭自动战斗
                if (this.autofightInterval) {
                    clearInterval(this.autofightInterval);
                    this.autofightInterval = null;
                }
            }
        },

        // ====================================================================
        // 异步命令执行 (使用请求队列，防止并发导致状态不一致)
        // ====================================================================

        /**
         * 异步发送命令（使用队列模式）
         * 优点：同一用户的请求串行执行，防止并发导致状态不一致
         * @param {string} cmd 要执行的命令
         * @param {number} timeout 超时时间(ms)，默认5000
         * @param {boolean} isRetry 是否为重试（防止无限循环）
         * @returns {Promise<string>} 返回HTML结果
         */
        async sendAsyncCommand(cmd, timeout = 5000, isRetry = false) {
            if (!this.txd) {
                throw new Error('未登录');
            }

            try {
                // 1. 发送异步请求
                const asyncResp = await fetch(`${this.apiBase}/api/async`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                    body: new URLSearchParams({
                        txd: this.txd,
                        cmd: cmd
                    })
                });

                if (!asyncResp.ok) {
                    // 401 会话过期，尝试重新登录并重试
                    if (asyncResp.status === 401 && !isRetry) {
                        console.log('[Async] 会话过期，尝试重新登录并重试...');
                        await this.relogin();
                        if (!this.showLogin) {
                            return this.sendAsyncCommand(cmd, timeout, true);
                        }
                    }
                    throw new Error(`API错误: ${asyncResp.status}`);
                }

                const asyncData = await asyncResp.json();

                if (asyncData.error) {
                    // 认证错误，尝试重新登录并重试
                    if ((asyncData.error.includes('认证') || asyncData.error.includes('登录') || asyncData.error.includes('未登录')) && !isRetry) {
                        console.log('[Async] 会话过期，尝试重新登录并重试...');
                        await this.relogin();
                        if (!this.showLogin) {
                            return this.sendAsyncCommand(cmd, timeout, true);
                        }
                    }
                    throw new Error(asyncData.error);
                }

                const requestId = asyncData.request_id;
                if (!requestId) {
                    throw new Error('未获得request_id');
                }

                console.log(`[Async] 命令已入队: ${cmd}, requestId: ${requestId}, 队列位置: ${asyncData.queue_position}`);

                // 2. 轮询结果
                const startTime = Date.now();
                while (Date.now() - startTime < timeout) {
                    await new Promise(resolve => setTimeout(resolve, 100)); // 100ms轮询间隔

                    const resultResp = await fetch(`${this.apiBase}/api/result?request_id=${encodeURIComponent(requestId)}&txd=${encodeURIComponent(this.txd)}`);

                    if (!resultResp.ok) {
                        // 401 会话过期，尝试重新登录并重试
                        if (resultResp.status === 401 && !isRetry) {
                            console.log('[Async] 会话过期，尝试重新登录并重试...');
                            await this.relogin();
                            if (!this.showLogin) {
                                return this.sendAsyncCommand(cmd, timeout, true);
                            }
                        }
                        throw new Error(`Result API错误: ${resultResp.status}`);
                    }

                    const contentType = resultResp.headers.get('content-type');
                    if (contentType && contentType.includes('application/json')) {
                        // JSON响应 - 还在处理中
                        const resultData = await resultResp.json();
                        if (resultData.status === 'pending') {
                            continue; // 继续轮询
                        }
                        if (resultData.error) {
                            // 认证错误，尝试重新登录并重试
                            if ((resultData.error.includes('认证') || resultData.error.includes('登录') || resultData.error.includes('未登录')) && !isRetry) {
                                console.log('[Async] 会话过期，尝试重新登录并重试...');
                                await this.relogin();
                                if (!this.showLogin) {
                                    return this.sendAsyncCommand(cmd, timeout, true);
                                }
                            }
                            throw new Error(resultData.error);
                        }
                    } else if (contentType && contentType.includes('text/html')) {
                        // HTML响应 - 完成
                        const html = await resultResp.text();
                        console.log(`[Async] 命令完成: ${cmd}, 耗时: ${Date.now() - startTime}ms`);
                        return html;
                    }
                }

                throw new Error('请求超时');

            } catch (e) {
                console.error(`[Async] 命令失败: ${cmd}`, e);
                throw e;
            }
        },

        /**
         * 使用异步模式发送快捷命令
         * 相比直接更新iframe.src，这种方式不会导致页面闪烁
         */
        async sendQuickCommandAsync(cmd) {
            this.lastCommand = cmd;
            this.frameLoading = true;

            try {
                const html = await this.sendAsyncCommand(cmd);
                // 更新iframe内容
                const iframe = this.$refs.gameFrame;
                if (iframe && iframe.contentWindow) {
                    iframe.contentWindow.document.open();
                    iframe.contentWindow.document.write(html);
                    iframe.contentWindow.document.close();
                }
            } catch (e) {
                console.error('异步命令执行失败:', e);
                // 降级到同步模式
                const url = `${this.apiBase}/api/html?txd=${encodeURIComponent(this.txd)}&cmd=${encodeURIComponent(cmd)}`;
                const iframe = this.$refs.gameFrame;
                if (iframe) {
                    iframe.src = url;
                }
            } finally {
                this.frameLoading = false;
            }
        },

        /**
         * 使用异步模式发送命令输入
         */
        async sendCommandAsync() {
            const cmd = this.commandInput.trim();
            if (!cmd) return;

            this.lastCommand = cmd;
            this.commandInput = '';
            this.showCommandInput = false;
            this.frameLoading = true;

            try {
                const html = await this.sendAsyncCommand(cmd);
                // 更新iframe内容
                const iframe = this.$refs.gameFrame;
                if (iframe && iframe.contentWindow) {
                    iframe.contentWindow.document.open();
                    iframe.contentWindow.document.write(html);
                    iframe.contentWindow.document.close();
                }
            } catch (e) {
                console.error('异步命令执行失败:', e);
                // 降级到同步模式
                const url = `${this.apiBase}/api/html?txd=${encodeURIComponent(this.txd)}&cmd=${encodeURIComponent(cmd)}`;
                const iframe = this.$refs.gameFrame;
                if (iframe) {
                    iframe.src = url;
                }
            } finally {
                this.frameLoading = false;
            }
        },

        // ==================== 战斗系统方法 ====================

        /**
         * 检测是否处于战斗状态
         * 通过检查 mudLines 中是否有"察看战况"按钮或"关闭自动战斗"按钮
         * 有"关闭自动战斗"按钮也说明处于战斗状态（虽然可能没有active round）
         */
        async checkBattleStatus() {
            const hasBattleButton = this.mudLines.some(line =>
                line.segments && line.segments.some(seg =>
                    seg.type === 'button' && (seg.label === '察看战况' || seg.label.includes('关闭自动战斗'))
                )
            );

            const wasInBattle = this.isInBattle;
            this.isInBattle = hasBattleButton;

            // 战斗状态变化时的处理
            if (hasBattleButton && !wasInBattle) {
                // 进入战斗
                console.log('[战斗] 进入战斗状态');
                this.battleLog = [];
                this.startBattleStatusPolling();
                // 立即获取一次战斗状态
                await this.fetchBattleStatus();
            } else if (!hasBattleButton && wasInBattle) {
                // 离开战斗
                console.log('[战斗] 离开战斗状态');
                this.battleEnemy = null;
                this.battleEnemyFull = null;
                this.battlePlayerFull = null;
                this.stopBattleStatusPolling();
            } else if (hasBattleButton) {
                // 仍在战斗中，更新状态
                await this.fetchBattleStatus();
            }
        },

        /**
         * 启动战斗状态轮询
         */
        startBattleStatusPolling() {
            this.stopBattleStatusPolling();
            // 每1秒轮询一次战斗状态
            this.battleStatusInterval = setInterval(() => {
                this.fetchBattleStatus();
            }, 1000);
        },

        /**
         * 停止战斗状态轮询
         */
        stopBattleStatusPolling() {
            if (this.battleStatusInterval) {
                clearInterval(this.battleStatusInterval);
                this.battleStatusInterval = null;
            }
        },

        /**
         * 获取战斗状态（敌我双方）
         */
        async fetchBattleStatus() {
            try {
                const response = await fetch(`${this.apiBase}/api/battle_status?txd=${encodeURIComponent(this.txd)}`);
                if (!response.ok) return;

                const data = await response.json();

                if (data.in_battle) {
                    // 更新玩家完整状态
                    if (data.player) {
                        this.battlePlayerFull = data.player;
                    }

                    // 更新敌人状态
                    if (data.enemy) {
                        this.battleEnemyFull = data.enemy;
                        this.battleEnemy = {
                            name: data.enemy.name_cn || data.enemy.name,
                            hp: data.enemy.hp,
                            hpMax: data.enemy.hp_max,
                            is_npc: data.enemy.is_npc
                        };
                    } else {
                        this.battleEnemyFull = null;
                        this.battleEnemy = null;
                    }
                }
            } catch (e) {
                console.error('[战斗] 获取战斗状态失败:', e);
            }
        },

        /**
         * 解析战斗信息（敌人名称、HP等）
         */
        parseBattleInfo() {
            // 从 mudLines 中查找战斗相关的信息
            // 常见格式: "你对XXX发动攻击" 或 "XXX正在攻击你"
            for (const line of this.mudLines) {
                if (!line.segments) continue;
                const lineText = line.segments.map(s => {
                    if (s.type === 'text') {
                        return s.parts ? s.parts.map(p => p.content || '').join('') : '';
                    }
                    return '';
                }).join('');

                // 解析敌人名称
                const enemyMatch = lineText.match(/对(.{2,6})发动攻击|(.{2,6})正在攻击|(.{2,6})对你造成|战胜了(.{2,6})/);
                if (enemyMatch) {
                    const enemyName = enemyMatch[1] || enemyMatch[2] || enemyMatch[3] || enemyMatch[4];
                    if (enemyName && !enemyName.includes('你') && !enemyName.includes('自动')) {
                        if (!this.battleEnemy || this.battleEnemy.name !== enemyName) {
                            this.battleEnemy = { name: enemyName, hp: null, hpMax: null };
                        }
                        break;
                    }
                }
            }
        },

        /**
         * 解析战斗动作并生成动画
         * @param {Array} newLines - 新的MUD输出行
         */
        parseBattleActions(newLines) {
            if (!this.isInBattle) return;

            for (const line of newLines) {
                if (!line.segments) continue;

                // 构建完整文本行（包含文本段和按钮标签）
                const lineText = line.segments.map(s => {
                    if (s.type === 'text') {
                        return s.parts ? s.parts.map(p => p.content || '').join('') : '';
                    }
                    return s.label || '';
                }).join('');

                // 跳过空行
                if (!lineText || lineText.trim().length === 0) continue;

                // 跳过纯按钮行（如"察看战况"）
                const isButtonOnly = line.segments.length === 1 && line.segments[0].type === 'button';
                if (isButtonOnly) continue;

                // 把所有战斗文本记录到日志
                const trimmedText = lineText.trim();
                if (trimmedText.length > 0 && trimmedText.length < 200) {
                    this.addBattleLog('info', trimmedText);
                }

                // 解析武侠技能类型并添加技能动画
                const skillType = this.parseMartialArtsSkill(lineText);
                if (skillType) {
                    this.addSkillAnimation(skillType);
                }

                // 解析特殊战斗状态
                if (lineText.match(/躲过.*攻击|闪避.*招式|身法.*避开/)) {
                    this.addSkillAnimation('dodge');
                }
                if (lineText.match(/格挡.*攻击|招架.*招式|成功.*防御/)) {
                    this.addSkillAnimation('block');
                }
                if (lineText.match(/身中剧毒|毒发.*伤|中毒.*发作/)) {
                    this.addSkillAnimation('poison');
                }

                // 解析伤害数字用于动画
                const damageToEnemyMatch = lineText.match(/(\d+)点.*?伤害/);
                if (damageToEnemyMatch && lineText.includes('你')) {
                    const damage = parseInt(damageToEnemyMatch[1] || 0);
                    if (damage > 0) {
                        const isPlayerAttacking = lineText.includes('你造成') || lineText.includes('你对');
                        const criticalMatch = lineText.match(/暴击|致命|会心一击/);
                        if (isPlayerAttacking) {
                            this.addBattleAnimation('damage', 'enemy', damage, criticalMatch);
                            if (criticalMatch) {
                                this.addSkillAnimation('critical');
                            }
                        } else {
                            this.addBattleAnimation('damage', 'player', damage, criticalMatch);
                        }
                    }
                }

                // 战斗胜利
                if (lineText.match(/战斗胜利|战胜了|击败|获胜/)) {
                    this.addBattleAnimation('victory', null, null);
                }

                // 解析敌人状态（HP显示）
                const hpMatch = lineText.match(/(.{2,6})[：:]\s*(\d+)\/(\d+)/);
                if (hpMatch) {
                    const name = hpMatch[1].trim();
                    const hp = parseInt(hpMatch[2]);
                    const hpMax = parseInt(hpMatch[3]);
                    if (!name.includes('你') && name.length >= 2 && name.length <= 6) {
                        this.battleEnemy = { name, hp, hpMax };
                    }
                }
            }
        },

        /**
         * 解析武侠技能类型
         * @param {string} text - 战斗文本
         * @returns {string|null} 技能类型
         */
        parseMartialArtsSkill(text) {
            // 剑类技能
            if (text.includes('剑气') || text.includes('剑芒') || text.includes('万剑') ||
                text.includes('独孤九剑') || text.includes('太极剑') || text.includes('神剑') ||
                text.includes('剑光') || text.includes('御剑') || text.includes('剑影')) {
                return 'sword-qi';
            }

            // 掌法技能
            if (text.includes('掌') && (text.includes('劈') || text.includes('击') || text.includes('拍')) ||
                text.includes('降龙十八掌') || text.includes('火焰掌') || text.includes('寒冰掌') ||
                text.includes('铁砂掌') || text.includes('摧心掌') || text.includes('幻阴掌') ||
                text.includes('大力金刚掌') || text.includes('七伤掌')) {
                return 'palm';
            }

            // 指法技能
            if (text.includes('指') && (text.includes('戳') || text.includes('点') || text.includes('击')) ||
                text.includes('六脉神剑') || text.includes('一阳指') || text.includes('幻阴指') ||
                text.includes('弹指神通') || text.includes('指力') || text.includes('指风')) {
                return 'finger';
            }

            // 拳法技能
            if (text.includes('拳') && (text.includes('轰') || text.includes('击') || text.includes('冲')) ||
                text.includes('七伤拳') || text.includes('太极拳') || text.includes('伏虎拳') ||
                text.includes('罗汉拳') || text.includes('长拳') || text.includes('崩拳') ||
                text.includes('铁拳') || text.includes('重拳')) {
                return 'fist';
            }

            // 轻功技能
            if (text.includes('轻功') || text.includes('身法') || text.includes('移形换位') ||
                text.includes('梯云纵') || text.includes('凌波微步') || text.includes('踏雪无痕') ||
                text.includes('飞檐走壁') || text.includes('瞬移') || text.includes('闪身')) {
                return 'lightness';
            }

            // 内功技能
            if (text.includes('内力') || text.includes('真气') || text.includes('内功') ||
                text.includes('九阳神功') || text.includes('九阴真经') || text.includes('易筋经') ||
                text.includes('紫霞神功') || text.includes('蛤蟆功') || text.includes('神照功') ||
                text.includes('混元功') || text.includes('龙象般若功') || text.includes('北冥神功')) {
                return 'inner-power';
            }

            // 棍棒技能
            if (text.includes('棒') || text.includes('棍') && (text.includes('扫') || text.includes('打')) ||
                text.includes('打狗棒法') || text.includes('降魔棍') || text.includes('齐眉棍') ||
                text.includes('棒影') || text.includes('横扫')) {
                return 'staff';
            }

            // 刀法技能
            if (text.includes('刀') && (text.includes('斩') || text.includes('劈') || text.includes('砍')) ||
                text.includes('玄铁重剑') || text.includes('血刀') || text.includes('刀光') ||
                text.includes('狂风刀') || text.includes('破刀') || text.includes('刀气') ||
                text.includes('胡家刀法') || text.includes('火焰刀')) {
                return 'saber';
            }

            return null;
        },

        /**
         * 添加武侠技能动画
         * @param {string} skillType - 技能类型
         */
        addSkillAnimation(skillType) {
            const id = 'skill-' + Date.now() + '-' + Math.random();
            const skillEffect = {
                id,
                type: skillType
            };

            // 添加到技能动画列表
            if (!this.skillAnimations) {
                this.skillAnimations = [];
            }
            this.skillAnimations.push(skillEffect);

            // 自动移除动画
            const duration = {
                'sword-qi': 1000,
                'palm': 800,
                'finger': 600,
                'fist': 500,
                'lightness': 1000,
                'inner-power': 1200,
                'staff': 700,
                'saber': 600,
                'critical': 800,
                'dodge': 500,
                'block': 600,
                'poison': 1500
            }[skillType] || 1000;

            setTimeout(() => {
                this.skillAnimations = this.skillAnimations.filter(s => s.id !== id);
            }, duration);
        },

        /**
         * 获取技能动画的CSS类名
         * @param {string} skillType - 技能类型
         * @returns {string} CSS类名
         */
        getSkillAnimationClass(skillType) {
            const classMap = {
                'sword-qi': 'skill-sword-qi',
                'palm': 'skill-palm-wave',
                'finger': 'skill-finger-strike',
                'fist': 'skill-fist-strike',
                'lightness': 'skill-lightness',
                'inner-power': 'skill-inner-power',
                'staff': 'skill-staff-sweep',
                'saber': 'skill-saber-slash',
                'critical': 'skill-critical-blow',
                'dodge': 'skill-dodge',
                'block': 'skill-block',
                'poison': 'skill-poison'
            };
            return classMap[skillType] || '';
        },

        /**
         * 获取技能图标
         * @param {string} skillType - 技能类型
         * @returns {string} 图标emoji
         */
        getSkillIcon(skillType) {
            const iconMap = {
                'sword-qi': '⚔️',
                'palm': '🖐️',
                'finger': '👆',
                'fist': '👊',
                'lightness': '💨',
                'inner-power': '✨',
                'staff': '🎋',
                'saber': '🗡️',
                'critical': '💥',
                'dodge': '💫',
                'block': '🛡️',
                'poison': '☠️'
            };
            return iconMap[skillType] || '⭐';
        },

        /**
         * 获取技能图标的CSS类名
         * @param {string} skillType - 技能类型
         * @returns {string} CSS类名
         */
        getSkillIconClass(skillType) {
            const classMap = {
                'sword-qi': 'sword-qi-icon',
                'palm': 'palm-wave-icon',
                'finger': 'finger-strike-icon',
                'fist': 'fist-strike-icon',
                'lightness': 'lightness-icon',
                'inner-power': 'inner-power-icon',
                'staff': 'staff-sweep-icon',
                'saber': 'saber-slash-icon',
                'critical': 'critical-blow-icon',
                'dodge': 'dodge-icon',
                'block': 'block-icon',
                'poison': 'poison-icon'
            };
            return classMap[skillType] || '';
        },

        /**
         * 添加战斗动画
         * @param {string} type - 动画类型: damage, heal, victory
         * @param {string} target - 目标: player, enemy
         * @param {number} value - 数值
         * @param {boolean} isCritical - 是否暴击
         */
        addBattleAnimation(type, target, value, isCritical = false) {
            const id = Date.now() + Math.random();
            const animation = { id, type, target, value, isCritical };
            this.battleAnimations.push(animation);

            // 自动移除动画
            setTimeout(() => {
                this.battleAnimations = this.battleAnimations.filter(a => a.id !== id);
            }, 2000);
        },

        /**
         * 添加战斗日志
         * @param {string} type - 日志类型
         * @param {string} message - 消息
         */
        addBattleLog(type, message) {
            const timestamp = new Date().toLocaleTimeString('zh-CN', { hour12: false });
            this.battleLog.unshift({ type, message, timestamp });
            // 只保留最近50条
            if (this.battleLog.length > 50) {
                this.battleLog = this.battleLog.slice(0, 50);
            }
        },

        /**
         * 切换迷你模式
         */
        toggleBattleMiniMode() {
            this.battleMiniMode = !this.battleMiniMode;
            localStorage.setItem('battle_mini_mode', this.battleMiniMode ? '1' : '0');
        },

        /**
         * 切换全屏模式
         */
        toggleBattleFullscreen() {
            this.battleFullscreen = !this.battleFullscreen;
            // 全屏时自动展开战斗日志
            if (this.battleFullscreen) {
                this.battleShowLog = true;
            }
        },

        /**
         * 切换战斗日志显示
         */
        toggleBattleLog() {
            this.battleShowLog = !this.battleShowLog;
        },

        /**
         * 清空战斗日志
         */
        clearBattleLog() {
            this.battleLog = [];
        },

        /**
         * 切换快捷菜单显示/隐藏
         */
        toggleQuickActions() {
            this.quickActionsCollapsed = !this.quickActionsCollapsed;
            localStorage.setItem('quickActionsCollapsed', this.quickActionsCollapsed ? '1' : '0');
        },

        /**
         * 根据内容行数动态调整容器高度
         */
        adjustContainerHeight() {
            const container = document.querySelector('.mud-output-container');
            if (!container) return;

            const lineCount = this.mudLines.length;
            // 每行约 14px 高度，几乎不留基础空间
            const lineHeight = 14;
            const baseHeight = 5;  // 几乎不留基础高度

            // 完全根据行数计算
            let calculatedHeight = baseHeight + (lineCount * lineHeight);

            container.style.minHeight = calculatedHeight + 'px';
            container.style.maxHeight = 'none';
        },

        /**
         * 打开招式列表
         */
        async openPerformsList() {
            this.performsLoading = true;
            this.showPerformsList = true;

            try {
                const txd = localStorage.getItem('mud_txd') || this.txd;
                if (!txd) {
                    alert('请先登录');
                    this.showPerformsList = false;
                    return;
                }

                const response = await fetch(`${this.apiBase}/api/performs?txd=${encodeURIComponent(txd)}`);
                const data = await response.json();

                if (data.error) {
                    console.error('获取招式列表失败:', data.error);
                    this.performsData = {
                        performs: [],
                        skill_name_cn: '错误',
                        message: data.error
                    };
                } else {
                    this.performsData = data;
                }
            } catch (e) {
                console.error('获取招式列表失败:', e);
                this.performsData = {
                    performs: [],
                    skill_name_cn: '错误',
                    message: '网络错误'
                };
            } finally {
                this.performsLoading = false;
            }
        },

        /**
         * 关闭招式列表
         */
        closePerformsList() {
            this.showPerformsList = false;
            this.performsData = null;
        },

        /**
         * 选择招式并执行
         */
        async selectPerform(perform) {
            if (!perform.available) {
                alert(`该招式需要武功等级达到 ${perform.level_req} 级`);
                return;
            }
            if (!perform.enough_neili) {
                alert(`内力不足！需要 ${perform.neili_cost} 点内力`);
                return;
            }

            // 发送 perform 命令
            await this.sendJsonCommand(`perform ${perform.id}`);

            // 关闭招式列表，但保持全屏战斗窗口
            this.closePerformsList();
        },

        /**
         * 检查是否有针对特定目标的伤害动画
         */
        hasDamageAnimation(target) {
            return this.battleAnimations.some(anim =>
                anim.target === target && anim.type === 'damage'
            );
        },

        /**
         * 获取动画的CSS类名
         */
        getAnimationClass(anim) {
            let classes = [];
            if (anim.type === 'damage') {
                classes.push('damage-animation');
                if (anim.target === 'player') classes.push('damage-to-player');
                else classes.push('damage-to-enemy');
            } else if (anim.type === 'heal') {
                classes.push('heal-animation');
            } else if (anim.type === 'victory') {
                classes.push('victory-animation');
            }
            if (anim.isCritical) {
                classes.push('critical-animation');
            }
            return classes.join(' ');
        }
    },

    computed: {
        // 将区号转换为可读格式 (tx01 -> 1区, tx02 -> 2区, etc.)
        areaName() {
            const partition = this.loginForm.partition || '';
            // 匹配 tx后跟数字的格式
            const match = partition.match(/^tx(\d+)/);
            if (match) {
                const areaNum = parseInt(match[1], 10);
                return areaNum + '区';
            }
            // 如果不是标准格式，返回原值
            return partition;
        }
    },

    mounted() {
        // 保存实例到全局以便HTML中的onclick调用
        window.vueInstance = this;

        this.apiBase = this.detectApiBase();
        const modeText = this.useJsonMode ? 'JSON模式 (无iframe)' : 'iframe模式';
        console.log(`Vue游戏客户端已启动 (${modeText})`);

        // 从localStorage恢复主题设置
        const savedTheme = localStorage.getItem('mud_theme');
        if (savedTheme) {
            this.theme = savedTheme;
        }
        this.applyTheme();

        // 恢复战斗迷你模式设置
        const savedMiniMode = localStorage.getItem('battle_mini_mode');
        if (savedMiniMode === '0') {
            this.battleMiniMode = false;
        } else {
            this.battleMiniMode = true;  // 默认迷你模式
        }

        // 恢复快捷菜单折叠状态
        const savedQuickActionsCollapsed = localStorage.getItem('quickActionsCollapsed');
        if (savedQuickActionsCollapsed === '1') {
            this.quickActionsCollapsed = true;
        }

        console.log('API地址:', this.apiBase);

        // 生成验证码
        this.refreshCaptcha();

        // 加载分区列表
        this.loadPartitions();

        // 添加页面可见性监听（用于移动端后台恢复）
        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                console.log('[页面恢复] 刷新状态并检查自动战斗');
                // 页面回到前台，立即刷新状态
                this.fetchPlayerStats();
                // 如果自动战斗定时器存在，确保它继续运行
                if (this.autofightInterval && this.playerStats && this.playerStats.autofight) {
                    console.log('[自动战斗] 确认定时器运行中');
                }
            }
        });

        // 尝试从 localStorage 恢复登录状态
        const savedTxd = localStorage.getItem('mud_txd');
        const savedPartition = localStorage.getItem('mud_partition');
        const savedUser = localStorage.getItem('mud_userid');

        if (savedTxd && savedUser) {
            // 有保存的登录信息，自动恢复
            this.txd = savedTxd;
            this.loginForm.partition = savedPartition || 'tx01';
            this.loginForm.userid = savedUser;

            console.log('恢复登录: txd=', savedTxd.substring(0, 20) + '...');
            console.log('apiBase=', this.apiBase);

            if (this.useJsonMode) {
                // JSON模式: 加载初始MUD输出
                this.showLogin = false;
                this.sendJsonCommand('init');
            } else {
                // iframe模式: 设置iframe URL
                this.gameFrameUrl = this.getGameFrameUrl();
                console.log('gameFrameUrl=', this.gameFrameUrl);
                this.showLogin = false;
            }

            // 开始更新玩家状态
            this.startStatsUpdate();
        } else {
            // 无保存的登录信息，恢复表单
            if (savedPartition) {
                this.loginForm.partition = savedPartition;
            }
            if (savedUser) {
                this.loginForm.userid = savedUser;
            }
        }
    }
}).mount('#app');

// 全局聊天链接点击处理器
window.handleChatLinkClick = function(element) {
    const command = element.getAttribute('data-command');
    if (command && window.vueInstance) {
        // 关闭聊天室
        window.vueInstance.closeChatRoom();
        // 在主界面执行命令显示装备
        window.vueInstance.sendQuickCommand(command);
    }
};

