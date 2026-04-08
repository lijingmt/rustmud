<!-- 翻译插件初始化 (Translate.js Integration) with Auto Language Detection -->
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

    // 隐藏底部语言选择器（右上角已有）
    translate.selectLanguageTag.show = false;

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

