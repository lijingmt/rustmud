/**
 * Vue游戏客户端
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
                navigation: null,
                actions: []
            },

            inputCommand: '',

            // API地址 - Connect to Rust HTTP API on port 8081
            apiBase: 'http://' + window.location.hostname + ':8081'
        };
    },

    computed: {
        connectionStatus() {
            return this.showLogin ? '未登录' : '已连接';
        }
    },

    methods: {
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

                // 发送第一个命令
                const result = await this.apiRequest('look');

                if (result.error) {
                    this.loginError = result.error;
                } else {
                    this.showLogin = false;
                    this.$nextTick(() => {
                        this.$refs.cmdInput?.focus();
                    });
                }
            } catch (e) {
                this.loginError = '连接失败: ' + e.message;
            } finally {
                this.isLoggingIn = false;
            }
        },

        // TXD编码 (复用do.jsp逻辑)
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
                    }
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
                        msg.id = 'msg_' + Date.now() + '_' + idx;
                        this.state.messages.push(msg);
                    });

                    // 滚动到底部
                    this.$nextTick(() => {
                        const container = this.$refs.messageContainer;
                        if (container) {
                            container.scrollTop = container.scrollHeight;
                        }
                    });
                }

                return data;
            } finally {
                this.isLoading = false;
            }
        },

        // 发送命令
        async sendCommand(cmd) {
            if (!cmd || this.isLoading) return;

            this.inputCommand = '';
            await this.apiRequest(cmd);
        },

        // 获取导航图标
        getNavIcon(label) {
            const icons = {
                '北方': '↑', '南': '↓', '东方': '→', '西方': '←',
                '上': '▲', '下': '▼',
                '北': '↑', '南↓': '↓', '东→': '→', '西←': '←'
            };
            return icons[label] || '●';
        }
    },

    mounted() {
        console.log('Vue游戏客户端已启动');
        console.log('API地址:', this.apiBase);

        // 检查本地存储的登录信息
        const savedUser = localStorage.getItem('mud_userid');
        if (savedUser) {
            this.loginForm.userid = savedUser;
        }
    }
}).mount('#app');
