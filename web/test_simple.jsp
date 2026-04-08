<%@ page contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@include file="header1.inc"%>
<html>
<head><title>Test Simple</title>
<link rel="icon" type="image/x-icon" href="images/favicon.ico">
</head>
<body>
Hello World
<!-- 翻译插件初始化 (Translate.js Integration) -->
<div class="translate-area">
  <div class="translate-wrapper">
    <script src="includes/translate.js"></script>
    <script>
      translate.language.setLocal('chinese_simplified');
      translate.service.use('client.edge');
      translate.setAutoDiscriminateLocalLanguage();
      translate.execute();
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

</body>
</html>
