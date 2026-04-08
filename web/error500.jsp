<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8" %>
<%@include file="header1.inc"%>
<%@include file="common.inc"%>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>服务器维护中 - 天下</title>
    <link href="includes/modern-gaming.css" rel="stylesheet" type="text/css"/>
    <link rel="icon" href="images/favicon.ico" type="image/x-icon">
    <style>
        .error-card {
            max-width: 700px;
            margin: 40px auto;
        }

        .error-code-display {
            font-size: 140px;
            font-weight: 900;
            background: linear-gradient(135deg, var(--primary-cyan) 0%, var(--primary-blue) 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            text-shadow: 0 0 30px rgba(0, 255, 255, 0.4);
            line-height: 1;
            margin: 20px 0;
            animation: float 3s ease-in-out infinite;
        }

        @keyframes float {
            0%, 100% { transform: translateY(0px); }
            50% { transform: translateY(-20px); }
        }

        .error-title {
            font-size: 32px;
            color: var(--primary-cyan);
            text-shadow: 0 0 20px rgba(0, 255, 255, 0.4);
            margin-bottom: 20px;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 15px;
        }

        .status-pulse {
            display: inline-block;
            width: 14px;
            height: 14px;
            background: #ffaa00;
            border-radius: 50%;
            box-shadow: 0 0 10px #ffaa00;
            animation: pulse 2s ease-in-out infinite;
        }

        @keyframes pulse {
            0%, 100% { opacity: 1; box-shadow: 0 0 10px #ffaa00; }
            50% { opacity: 0.6; box-shadow: 0 0 20px #ffaa00; }
        }

        .error-message {
            font-size: 16px;
            color: var(--text-secondary);
            line-height: 1.8;
            margin-bottom: 30px;
            padding: 20px;
            background: rgba(0, 255, 255, 0.05);
            border-left: 4px solid var(--primary-cyan);
            border-radius: 8px;
        }

        .maintenance-info {
            background: rgba(255, 170, 0, 0.1);
            border: 2px solid rgba(255, 170, 0, 0.3);
            border-radius: 12px;
            padding: 25px;
            margin: 30px 0;
            text-align: center;
        }

        .maintenance-info h3 {
            color: #ffaa00;
            font-size: 18px;
            margin-bottom: 12px;
            text-shadow: 0 0 10px rgba(255, 170, 0, 0.3);
        }

        .maintenance-info p {
            color: var(--text-secondary);
            font-size: 14px;
            line-height: 1.6;
            margin: 8px 0;
        }

        .button-group {
            display: flex;
            gap: 15px;
            justify-content: center;
            flex-wrap: wrap;
            margin: 30px 0;
        }

        .btn-block {
            width: 100%;
            text-align: center;
        }

        .loading-indicator {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 12px;
            margin: 30px 0;
            color: var(--text-secondary);
            font-size: 14px;
        }

        .spinner {
            display: inline-block;
            width: 30px;
            height: 30px;
            border: 3px solid rgba(0, 255, 255, 0.2);
            border-top-color: var(--primary-cyan);
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }

        @keyframes spin {
            to { transform: rotate(360deg); }
        }

        .error-timestamp {
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid rgba(0, 255, 255, 0.2);
            color: var(--text-secondary);
            font-size: 12px;
        }

        .error-timestamp p {
            margin: 5px 0;
        }

        @media (max-width: 600px) {
            .error-card {
                margin: 20px auto;
            }

            .error-code-display {
                font-size: 100px;
            }

            .error-title {
                font-size: 24px;
            }

            .button-group {
                flex-direction: column;
            }

            .btn {
                width: 100%;
            }
        }
    </style>
</head>
<body class="classical-pattern">
    <div class="header">
        <div class="container-narrow">
            <img src="logo-1.gif" class="logo">
            <div class="header-title">《天下AI网游》</div>
            <div class="header-subtitle">原版天下 • 醇香文字 • 邀你同行</div>
        </div>
    </div>

    <div class="container-narrow">
        <div class="card error-card">
            <div style="text-align: center;">
                <div class="error-code-display">500</div>
                <div class="error-title">
                    <span class="status-pulse"></span>
                    服务器维护中
                </div>
            </div>

            <div class="error-message">
                <p>游戏服务器暂时无法连接，可能正在进行例行维护或升级。</p>
                <p>我们的技术团队正在努力恢复服务，感谢您的耐心等待！</p>
            </div>

            <div class="loading-indicator">
                <span class="spinner"></span>
                <span>服务恢复中...</span>
            </div>

            <div class="maintenance-info">
                <h3>⚙️ 维护信息</h3>
                <p>如果问题持续存在，请联系游戏管理团队</p>
                <p>或访问官方网站获取最新维护公告</p>
            </div>

            <div class="button-group">
                <a href="javascript:window.location.reload()" class="btn btn-primary btn-block">
                    🔄 刷新页面
                </a>
                <a href="/" class="btn btn-secondary btn-block">
                    🏠 返回首页
                </a>
            </div>

            <div class="error-timestamp">
                <p>📍 错误代码: 500 - Internal Server Error</p>
                <p>⏰ 时间: <%= new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss").format(new java.util.Date()) %></p>
            </div>
        </div>
    </div>

    <div class="footer">
        <div class="footer-text">
            © 2025 天下AI网游
        </div>
    </div>
</body>
</html>
