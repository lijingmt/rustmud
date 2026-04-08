<!-- 翻译插件初始化 (Translate.js Integration) with Auto Language Detection -->
<div class="translate-area">
  <div class="translate-wrapper">
    <script src="includes/translate.js"></script>
    <script>
      // 自动检测浏览器语言并切换
      (function() {
        // 获取浏览器语言
        var browserLang = (navigator.language || navigator.userLanguage || 'zh-CN').toLowerCase();

        // 判断是否是英文
        var isEnglish = browserLang.indexOf('en') === 0;

        // 设置源语言（中文为默认）
        translate.language.setLocal('chinese_simplified');
        translate.service.use('client.edge');

        // 如果浏览器是英文，自动切换到英文
        if (isEnglish) {
          translate.language.setPageLanguage('english');
          console.log('自动切换到英文 (Browser Language: ' + browserLang + ')');
        } else {
          console.log('保持中文 (Browser Language: ' + browserLang + ')');
        }

        // 自动检测并应用本地语言
        translate.setAutoDiscriminateLocalLanguage();
        translate.execute();
      })();
    </script>
  </div>
</div>

<style>
.translate-area {
  width: 100%;
  height: 50px;
  display: flex;
  justify-content: center;
  align-items: center;
  background: transparent;
  margin-top: 20px;
}
.translate-wrapper {
  max-width: 200px;
  padding: 5px;
  font-size: 11px;
  color: #888;
  text-align: center;
}
.translate-wrapper select {
  font-size: 11px;
  padding: 2px 4px;
  max-width: 180px;
}
</style>
