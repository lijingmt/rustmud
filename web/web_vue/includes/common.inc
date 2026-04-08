<%@ page contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%@ page import="java.net.URL,
	java.net.HttpURLConnection,
	javax.naming.*,
	java.sql.Connection,
	javax.sql.DataSource,
	java.sql.PreparedStatement,
	java.io.*"
%>
	
<%!
String exec_pike(String ...args) {
	try {
		java.util.LinkedList li= new java.util.LinkedList(java.util.Arrays.asList(args));
		li.add(0,"/usr/local/bin/pike");
		ProcessBuilder builder = new ProcessBuilder(li);
		builder.redirectErrorStream(true);
		Process process = builder.start();
		InputStream ins = process.getInputStream();
		BufferedReader br = new BufferedReader(new InputStreamReader(ins,"GB2312"));
		String result = "";
		String line;
		while ((line = br.readLine()) != null) {
			result += line;
		}
		return result;
	} catch (Throwable e) {
		return "0";
	}
}
%>
