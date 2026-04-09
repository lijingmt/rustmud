/**
 * Vue游戏客户端 - 完整复刻txpike9
 * @version 2.0.0
 */
const { createApp } = Vue;

createApp({
    data() {
        return {
            showLogin: true,
            isLoggingIn: false,
            isLoading: false,
            loginError: '',
            loginForm: {
                userid: '',
                password: ''
            },

            // txd: 加密的认证信息
            txd: '',

            // 游戏状态
            state: {
                messages: [],
                player: null,
                navigation: { exits: [] },
                actions: []
            },

            inputCommand: '',
            commandHistory: [],
            historyIndex: -1,

            // API地址 - Connect to Rust HTTP API on port 8081
            apiBase: 'http://' + window.location.hostname + ':8081',

            // 自动刷新计时器
            refreshTimer: null,

            // 连接状态
            connected: false
        };
    },

    computed: {
        connectionStatus() {
            return this.showLogin ? '未连接' : '已连接';
        }
    },

    methods: {
        // 聚焦密码框
        focusPassword() {
            this.$refs.passwordInput?.focus();
        },

        // 登录 - 生成txd并发送第一个请求
        async doLogin() {
            if (!this.loginForm.userid || !this.loginForm.password) {
                this.loginError = '请输入账号和密码';
                return;
            }

            this.isLoggingIn = true;
            this.loginError = '';

            try {
                // 生成txd (与do.jsp相同的加密逻辑)
                this.txd = this.encodeTxd(this.loginForm.userid, this.loginForm.password);

                // 保存用户名到本地存储
                localStorage.setItem('mud_userid', this.loginForm.userid);
                localStorage.setItem('mud_txd', this.txd);

                // 发送第一个命令
                const result = await this.apiRequest('look');

                if (result.error) {
                    this.loginError = result.error;
                } else {
                    this.showLogin = false;
                    this.connected = true;
                    this.$nextTick(() => {
                        this.$refs.cmdInput?.focus();
                        this.scrollToBottom();
                    });
                }
            } catch (e) {
                this.loginError = '连接失败: ' + e.message;
            } finally {
                this.isLoggingIn = false;
            }
        },

        // TXD编码 (复刻do.jsp逻辑)
        encodeTxd(userid, password) {
            let uid = '';
            let pid = '';

            // 编码userid
            for (let i = 0; i < userid.length; i++) {
                let code = userid.charCodeAt(i);
                if (Math.floor(i / 2) === 0) {
                    // 偶数位: +2
                    if (code === 121) {
                        uid += '%7B';
                    } else {
                        uid += String.fromCharCode(code + 2);
                    }
                } else {
                    // 奇数位: +1
                    if (code === 122) {
                        uid += '%7B';
                    } else {
                        uid += String.fromCharCode(code + 1);
                    }
                }
            }

            // 编码password
            for (let i = 0; i < password.length; i++) {
                let code = password.charCodeAt(i);
                if (Math.floor(i / 2) === 0) {
                    // 偶数位: +1
                    if (code === 122) {
                        pid += '%7B';
                    } else {
                        pid += String.fromCharCode(code + 1);
                    }
                } else {
                    // 奇数位: +2
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

        // API请求
        async apiRequest(cmd) {
            this.isLoading = true;

            try {
                const params = new URLSearchParams({
                    txd: this.txd,
                    cmd: cmd
                });

                const response = await fetch(this.apiBase + '/api?' + params.toString(), {
                    method: 'GET',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    cache: 'no-store'
                });

                if (!response.ok) {
                    throw new Error('HTTP ' + response.status);
                }

                const data = await response.json();

                // 更新状态
                if (data.player) {
                    this.state.player = data.player;
                }
                if (data.navigation) {
                    this.state.navigation = data.navigation;
                }
                if (data.actions) {
                    this.state.actions = data.actions;
                }
                if (data.messages && Array.isArray(data.messages)) {
                    // 只添加新消息
                    const currentCount = this.state.messages.length;
                    data.messages.forEach((msg, idx) => {
                        msg.id = 'msg_' + Date.now() + '_' + idx + '_' + Math.random();
                        this.state.messages.push(msg);
                    });

                    // 滚动到底部
                    this.$nextTick(() => {
                        this.scrollToBottom();
                    });
                }

                return data;
            } catch (e) {
                console.error('API error:', e);
                // 添加错误消息
                this.state.messages.push({
                    id: 'err_' + Date.now(),
                    type: 'error',
                    text: '网络错误: ' + e.message
                });
                return { error: e.message };
            } finally {
                this.isLoading = false;
            }
        },

        // 发送命令
        async sendCommand(cmd) {
            if (!cmd || this.isLoading) return;

            // 添加到历史记录
            this.commandHistory.push(cmd);
            this.historyIndex = this.commandHistory.length;

            // 显示用户输入的命令
            this.state.messages.push({
                id: 'cmd_' + Date.now(),
                type: 'info',
                text: '> ' + cmd
            });

            this.inputCommand = '';
            await this.apiRequest(cmd);
        },

        // 滚动到底部
        scrollToBottom() {
            const container = this.$refs.messageContainer;
            if (container) {
                container.scrollTop = container.scrollHeight;
            }
        },

        // 检查是否有某个出口
        hasExit(direction) {
            if (!this.state.navigation?.exits) return false;
            return this.state.navigation.exits.some(exit => exit.direction === direction);
        },

        // 获取出口标签
        getExitLabel(direction) {
            if (!this.state.navigation?.exits) return direction;
            const exit = this.state.navigation.exits.find(e => e.direction === direction);
            return exit ? exit.label : direction;
        },

        // 处理方向键
        handleDirectionKey(event) {
            const keyMap = {
                'ArrowUp': 'north',
                'ArrowDown': 'south',
                'ArrowLeft': 'west',
                'ArrowRight': 'east',
                'w': 'north',
                's': 'south',
                'a': 'west',
                'd': 'east'
            };

            const cmd = keyMap[event.key];
            if (cmd && this.hasExit(cmd)) {
                event.preventDefault();
                this.sendCommand(cmd);
            }
        },

        // 命令历史导航
        handleHistoryNavigation(event) {
            if (event.key === 'ArrowUp') {
                event.preventDefault();
                if (this.historyIndex > 0) {
                    this.historyIndex--;
                    this.inputCommand = this.commandHistory[this.historyIndex];
                }
            } else if (event.key === 'ArrowDown') {
                event.preventDefault();
                if (this.historyIndex < this.commandHistory.length - 1) {
                    this.historyIndex++;
                    this.inputCommand = this.commandHistory[this.historyIndex];
                } else {
                    this.historyIndex = this.commandHistory.length;
                    this.inputCommand = '';
                }
            }
        },

        // 登出
        logout() {
            this.showLogin = true;
            this.connected = false;
            this.state.messages = [];
            this.state.player = null;
            this.txd = '';
            localStorage.removeItem('mud_txd');
        },

        // 渲染MUD颜色代码 (§Y=黄色, §R=红色, §G=绿色, §C=青色, §H=高亮, §N=正常)
        renderColorCodes(text) {
            if (!text) return '';

            const colorMap = {
                '§Y': '<span style="color: #FFD700;">',  // 黄色
                '§R': '<span style="color: #DC143C;">',  // 红色
                '§G': '<span style="color: #32CD32;">',  // 绿色
                '§C': '<span style="color: #00CED1;">',  // 青色
                '§W': '<span style="color: #FFFFFF;">',  // 白色
                '§B': '<span style="color: #4169E1;">',  // 蓝色
                '§M': '<span style="color: #FF69B4;">',  // 粉色
                '§O': '<span style="color: #FF8C00;">',  // 橙色
                '§H': '<span style="font-weight: bold; color: #FFD700;">',  // 高亮金色
                '§L': '<span style="color: #9370DB;">',  // 紫色
                '§N': '</span>',  // 正常（结束颜色）
                '§P': '<span style="color: #808080;">',  // 灰色
                '§Z': '<span style="color: #2F4F4F;">',  // 深青灰
            };

            let result = text;
            // 替换颜色代码
            for (const [code, html] of Object.entries(colorMap)) {
                result = result.split(code).join(html);
            }

            return result;
        },

        // 格式化消息（带颜色代码）
        formatMessage(msg) {
            const text = msg.text || '';
            const type = msg.type || 'info';

            // 根据消息类型添加样式
            let prefix = '';
            if (type === 'error') {
                prefix = '<span style="color: #DC143C;">[错误]</span> ';
            } else if (type === 'combat') {
                prefix = '<span style="color: #FF4500;">[战斗]</span> ';
            } else if (type === 'system') {
                prefix = '<span style="color: #00CED1;">[系统]</span> ';
            } else if (type === 'success') {
                prefix = '<span style="color: #32CD32;">[成功]</span> ';
            }

            return prefix + this.renderColorCodes(text);
        }
    },

    mounted() {
        console.log('《天下AI网游》Vue客户端已启动 v2.0.0');
        console.log('API地址:', this.apiBase);

        // 检查本地存储的登录信息
        const savedUser = localStorage.getItem('mud_userid');
        const savedTxd = localStorage.getItem('mud_txd');

        if (savedUser) {
            this.loginForm.userid = savedUser;
        }

        if (savedTxd) {
            this.txd = savedTxd;
        }

        // 添加键盘事件监听
        document.addEventListener('keydown', (e) => {
            if (this.showLogin) return;

            // 方向键移动
            if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'w', 'a', 's', 'd'].includes(e.key)) {
                if (document.activeElement !== this.$refs.cmdInput) {
                    this.handleDirectionKey(e);
                }
            }

            // 命令历史
            if (document.activeElement === this.$refs.cmdInput) {
                if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
                    this.handleHistoryNavigation(e);
                }
            }
        });

        // 快捷键
        document.addEventListener('keydown', (e) => {
            if (this.showLogin) return;

            // Ctrl/Cmd + L: look
            if ((e.ctrlKey || e.metaKey) && e.key === 'l') {
                e.preventDefault();
                this.sendCommand('look');
            }

            // Ctrl/Cmd + I: inventory
            if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
                e.preventDefault();
                this.sendCommand('inventory');
            }

            // Ctrl/Cmd + S: score
            if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                e.preventDefault();
                this.sendCommand('score');
            }

            // Esc: 聚焦命令输入框
            if (e.key === 'Escape') {
                this.$refs.cmdInput?.focus();
            }
        });

        // 页面可见性变化处理
        document.addEventListener('visibilitychange', () => {
            if (!document.hidden && !this.showLogin && this.txd) {
                // 页面重新可见时，刷新游戏状态
                this.apiRequest('look');
            }
        });

        // 定时刷新 (每30秒)
        this.refreshTimer = setInterval(() => {
            if (!this.showLogin && !this.isLoading) {
                // 静默刷新，不添加消息
                // this.apiRequest('score');
            }
        }, 30000);
    },

    beforeUnmount() {
        // 清理定时器
        if (this.refreshTimer) {
            clearInterval(this.refreshTimer);
        }
    }
}).mount('#app');
