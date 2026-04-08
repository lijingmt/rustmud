<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@include file="header1.inc"%>
<%@ page import="java.net.*,java.util.*,java.io.*" %>
<!DOCTYPE html>
<html>
<head>

 

	<meta charset="UTF-8">
	<meta name="viewport" content="maximum-scale=1.0,minimum-scale=1.0,user-scalable=0,width=device-width,initial-scale=1.0"/>
<title>《天下原版网游》[一区]</title>
<link href="includes/intro.css" rel="stylesheet" type="text/css"/>
    <%=favicon%>
</head>
<body>
<div>
	<!DOCTYPE html>
	<html lang="zh-cn">
		<head>
			<meta charset="UTF-8">
			<meta name="viewport" content="maximum-scale=1.0,minimum-scale=1.0,user-scalable=0,width=device-width,initial-scale=1.0"/>
			<title>《天下原版网游》[一区]</title>
			<link href="includes/bootstrap.min.css" rel="stylesheet">
			<style>
				body {
					font: Normal 18px "Noto Sans SC Medium";
					text-align: center;
				}
				@media (min-width: 768px) {
					body {
						margin: 0 auto;
						width: 414px;
					}
				}
				.logo {
					margin-top: 5px;
					width: 240px;
					height: 64px;
				}
				.form-group{
					width:90%;
					margin:5%;
				}
				.lable{
					width:60px;
					font-size: 16px;
					font-weight: 500;
					text-align: center;
				}
				.input{
					width:calc(100% - 80px);
					font-size: 16px;
				}
				.desc{
					width: 90%;
					margin: 5%;
				}
				.copyright{
					margin-top:30px;
				}
				.btns{
					margin-top:30px;
				}
				.btn{
					margin: 0 20px;
				}
				h2{
					font-size:20px;
				}
			</style>
		    <%=favicon%>
</head>
	<body>
  <%
  String z = (String)request.getParameter("z");

if(z==null)
	z=(String)request.getSession().getAttribute("z");
	else
	request.getSession().setAttribute("z",z);

	String user = request.getParameter("_user");
	String pswd = request.getParameter("_pswd");
	String regnewFlag = (String)request.getParameter("regnewFlag");  

	 String m_key = (String)session.getAttribute("m_key");   


if( user==null || pswd==null)
{

	//out.print("用户名和密码不能为空，请修改后重试。 <br/>");	
	response.sendRedirect("./pc_dark.jsp?err=1&"+paraString);
}
else
{
	user = user.trim();
	pswd = pswd.trim();

	String user_url = java.net.URLEncoder.encode(user,"UTF-8");
	String pswd_url = java.net.URLEncoder.encode(pswd,"UTF-8");

	if(user.length() == 0 || pswd.length() == 0)
	{
		//	out.print("|"+user + "|:|" + pswd + "|<br/>");
		//	out.print("用户名和密码不能为空，请修改后重试。 <br/>");	
		response.sendRedirect("./pc_dark.jsp?_user="+user+"&_pswd="+pswd+"&err=1&"+paraString);
	}
	else if( user.length()<2 || pswd.length()<2 )
	{
		//out.print("|"+user + "|:|" + pswd + "|<br/>");
		//out.print("为了你的安全，用户名和密码不能少于2个字符，请修改后重试。 <br/>");	
		response.sendRedirect("./pc_dark.jsp?_user="+user+"&_pswd="+pswd+"&err=2&"+paraString);
	}
	else if( user.length()>12 || pswd.length()>12 )
	{
		//resultData += "游戏账号和密码必须是2~12位的英文或者数字，或者两者的组合<br/>";
		response.sendRedirect("./pc_dark.jsp?_user="+user+"&_pswd="+pswd+"&err=5&"+paraString);
	}
	else
	{	
		String user_pswd = user + pswd;

		if(!isLegalChar(user_pswd))
		{
			//out.print("|"+user + "|:|" + pswd + "|<br/>");
			//out.print("用户名和密码只能是大小写字母或数字，请修改后重试。 <br/>");	
			response.sendRedirect("./pc_dark.jsp?_user="+user_url+"&_pswd="+pswd_url+"&err=3&"+paraString);
		}
		else 
		{
			Socket socket = new Socket(ip,port);
			InputStream reader = socket.getInputStream();
			OutputStream writer = socket.getOutputStream();

			String sid = (String)session.getId();
			String user_new = game_pre+user;
			send(writer,("login_check3 "+projname+" "+user_new+" "+pswd+" "+sid).getBytes());
			send(writer,"flush_filter".getBytes());
			socket.shutdownOutput();

			String ret = read(reader,"utf-8");

			if(writer!=null) writer.close();
			if(reader!=null) reader.close();
			if(socket!=null) socket.close();

			if(ret.equals("error1"))
			{
				//密码错误，或者两次登陆的session不同
				response.sendRedirect("./pc_dark.jsp?_user="+user+"&_pswd="+pswd+"&err=4&"+paraString);
			}
			else if(ret.equals("error2"))
			{
				//title += "您输入的用户名不存在，是否要注册这个帐户?\n";
				response.sendRedirect("./regnew.jsp?_user="+user+"&_pswd="+pswd+"&err=6&"+paraString);
			}
			else if(ret.equals("error3"))
			{
				//title += "您输入的用户名和密码认证失败，是否需要找回密码？\n";
				response.sendRedirect("./pc_dark.jsp?_user="+user+"&_pswd="+pswd+"&err=6&"+paraString);
			}
			else if(ret.equals("error4"))
			{
				 //严重错误，前台没有严格控制传入合法的用户名密码  
				response.sendRedirect("./pc_dark.jsp?_user="+user+"&_pswd="+pswd+"&err=7&"+paraString);
			}
			else
			{
			   response.sendRedirect("./do_dark.jsp?_user="+user_new+"&_pswd="+pswd+"&regnewFlag="+regnewFlag);   
			}
		}
	}
}
%>
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
		<script src="includes/jquery.min.js"></script>
		<script src="includes/bootstrap.min.js"></script>
		</html>
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
