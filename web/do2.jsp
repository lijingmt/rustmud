<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8" import="java.net.*,java.util.*,java.io.*" %>
<%@include file="header1.inc"%>
<%!
String jspname="do.jsp";
//String jspname="tx/do.jsp";
//String filter_type="wml522";
String filter_type="htmlcss_antirobot";
String title = gamename_cn;


String read(InputStream reader) throws IOException
{
    //BufferedReader r = new BufferedReader(new InputStreamReader(reader,"gb2312"));
    BufferedReader r = new BufferedReader(new InputStreamReader(reader,"GB18030"));
	String ret ="";
    String s = "";
	int n;
	try{
		s=r.readLine();
		while(s!=null&&!s.equals("")){//���while����������ܹ��죬�������ȥ����������Ϸҳ�����Ϸ�����һ�ж�����ַ�
			int i;
			for(i=0;i<s.length()&&s.charAt(i)!='|';i++);
			if(i!=s.length()){
			}
			s=r.readLine();
		}
		char buff[]=new char[4096];
		n=r.read(buff,0,4096);
		while(n!=-1){
			ret+=new String(buff,0,n);
			n=r.read(buff,0,4096);
		}
	}
	catch(Exception e){
		e.printStackTrace();
	}
	return ret;
}
%>
<%
try
{
	//request.setCharacterEncoding("gbk");
	request.setCharacterEncoding("UTF-8");
}catch(Exception uae)
{
	//request.setCharacterEncoding("utf-8");
	//com.lj.bbs.tools.setDefaultCharSet(request);
	System.out.println("[ua exception]"+uae+"\n");
}
String data=null;

HttpSession isession=null;
isession = request.getSession();

response.addHeader("Expires","Mon, 26 Jul 1997 05:00:00 GMT");
response.addHeader("Last-Modified","2004:08:05"+"GMT");	
response.addHeader("Cache-Control","no-cache, must-revalidate");
response.addHeader("Pragma","no-cache");

String userid = (String)request.getParameter("_user");
String passwd = (String)request.getParameter("_pswd");
//����ע��/////////////////////////////////////////////////////////////////////////////////////////////
String regnewFlag = (String)request.getParameter("regnewFlag");
String SID = (String)isession.getId();
if(regnewFlag!=null&&regnewFlag.equals("1"))
{
	response.sendRedirect("./login_reg.jsp?_user="+userid+"&_pswd="+passwd+"&sid="+SID+"&"+paraString);
	return;
}
///////////////////////////////////////////////////////////////////////////////////////////////////////
String txd=(String)request.getParameter("_txd");
String usid=(String)request.getParameter("_usid");
	//System.out.println("[txd=]["+txd+"]\n");
	//System.out.println("[usid=]["+usid+"]\n");
String uid="";
String pid="";
boolean first_login=false;
Socket socket;
InputStream reader;
OutputStream writer;

socket = new Socket(ip,port);
reader = socket.getInputStream();
writer = socket.getOutputStream();

if(userid!=null&&passwd!=null)//��һ�δ�login.jsp��½����_user��_pswd
{
	uid = userid;
	pid = passwd;
	first_login = true;
}
if(txd!=null&&!txd.equals("")&&!txd.equals(" "))
{
	String stru="";
	String strp="";
	int i=0;
	for(i=0;i<txd.length()&&txd.charAt(i)!='~';i++)
		;
	if(i!=txd.length())
	{
		stru = txd.substring(0,i);
		strp = txd.substring(i+1,txd.length());
	}
	//uid = stru;
	//pid = strp;
	//filter=html ����Ȳ��ӽ���
	for(int m=0; m<stru.length(); m++){
		char u;
		if(m/2==0)
			u = (char)(stru.charAt(m)-2);
		else
			u = (char)(stru.charAt(m)-1);
		String tmp = String.valueOf(u);
		uid += tmp; 
	}	
	for(int n=0; n<strp.length(); n++){
		char p;
		if(n/2==0)
			p = (char)(strp.charAt(n)-1);
		else
			p = (char)(strp.charAt(n)-2);
		String tmp = String.valueOf(p);
		pid += tmp;
	}
}

	//����ǰ�ر�
	/*
	if(uid.equals("tx01jinghaha")||uid.equals("tx0120230706")||uid.equals("tx01tqnu9410")||uid.equals("tx01lynnoo7")||uid.equals("tx01twjl6581"))
		;
	else
		response.sendRedirect("./wait10.jsp");
	*/

String cmd=request.getParameter("_cmd");
if( cmd!=null ){               
	cmd = cmd.trim();
	if( cmd.length()>4)
		cmd = "look";
	try{
		int i_cmd = Integer.parseInt(cmd);
	}
	catch(NumberFormatException e)
	{
		cmd = "look";
	}
}      
String arg=request.getParameter("arg");
//String temptitle=new String(title.getBytes("ISO8859-1"),"gb2312");
String temptitle=new String(title.getBytes("ISO8859-1"),"UTF-8");
//��һ����Ҳ��ÿ�ζ�Ҫ���͵�ָ��
//send(writer,("set_filter "+filter_type+" "+response.encodeURL("/"+jspname)+" "+title).getBytes("ISO8859-1"));
send(writer,("set_filter "+filter_type+" "+response.encodeURL("./"+jspname)+" "+title).getBytes("UTF-8"));

if(first_login){	
	//ʱ���������
	long ot = System.currentTimeMillis();
	Long time_ot = new Long(ot);
	isession.setAttribute("ot",time_ot);

	String _reg = (String)request.getParameter("_reg");
	if(_reg!=null&&_reg.equals("1")){
		String _sid = (String)request.getParameter("_sid");
		send(writer,("login_check1 "+projname+" "+userid+" "+passwd+" "+_sid).getBytes());
	}
	else{
		String userSessionID = (String)isession.getId();
		send(writer,("login_check1 "+projname+" "+userid+" "+passwd+" "+userSessionID).getBytes());
	}
}
else{
	//����������������
	Long otTmp = (Long)isession.getAttribute("ot");
	if(otTmp==null){
		long ot2 = System.currentTimeMillis()-1550;
		Long time_ot2 = new Long(ot2);
		otTmp = time_ot2;
		isession.setAttribute("ot",time_ot2);
	}
	long nt = System.currentTimeMillis();
	Long time_nt = new Long(nt);
	long otime = otTmp.longValue();
	long ntime = time_nt.longValue();
	long diffTime = ntime - otime; 
	if(diffTime<=1500){
		isession.setAttribute("ot",time_nt);
	}else
		isession.setAttribute("ot",time_nt);
	//System.out.println("[login uid=]["+uid+"]\n");
	//System.out.println("[login pid=]["+pid+"]\n");
	//System.out.println("[login usid=]["+usid+"]\n");
	send(writer,("login "+projname+" "+uid+" "+pid+" "+usid).getBytes());
}

if(first_login){
	String sid="hihi";
	send(writer,("set_sid "+sid).getBytes());
	String _mkey = (String)request.getParameter("_mkey");
	if(_mkey!=null)
		send(writer,("set_mkey"+" "+_mkey).getBytes()); 
}
if(cmd!=null&&cmd.equals("quit")){
	isession.removeAttribute("ot");
}

//System.out.println("****    [cmd=]["+cmd+"]\n");
if(cmd==null){
//System.out.println("html ***** [cmd=null] change cmd = init\n");
	cmd="init";
}

String _arg="";
//String _arg=request.getParameter("_arg");
//System.out.println("[_arg=]["+_arg+"]\n");
boolean have_space=false;
for(Enumeration en=request.getParameterNames();en.hasMoreElements();)
{
	String name = (String)en.nextElement();
	String value = request.getParameter(name);
	//System.out.println("------[name=]["+name+"]------");
	//System.out.println("------[value=]["+value+"]------");
	if("t".equals(name)) continue;

	//new///////////////////////////////
	if("arg".equals(name))
    	continue;
	if("_arg".equals(name))
	{
	    _arg = " "+value;
	}
	//new///////////////////////////////

	if(name.charAt(0)!='_'&&(name.length()<5)){
		cmd+=" "+name+"=";
	for(int i=0;i<value.length();i++){
		if(value.charAt(i)==' '){
			cmd+="%20";
		}
		else if(value.charAt(i)=='%'){
			cmd+="%%";
		}
		else{
			cmd+=value.substring(i,i+1);
		}
	}
	have_space=true;
	}
	//////////////////////////
	
}

if(arg!=null){
	String t_cmd="";
	for(int i=0;i<arg.length();i++){
		if(arg.charAt(i)==' '){
			t_cmd+="%20";
		}
		else if(arg.charAt(i)=='%'){
			t_cmd+="%%";
		}
		else{
			t_cmd+=arg.substring(i,i+1);
		}
	}
	cmd=cmd+" "+t_cmd;
	cmd=cmd.replaceAll("  "," ");
}
else if(have_space){
	cmd=cmd.trim();
}
if(_arg!=null){
    cmd=cmd+_arg;
}
//send(writer,cmd.getBytes("UTF-8"));
//System.out.println("[send cmd=]["+cmd+"]\n");
//send(writer,cmd.getBytes("gb2312"));
send(writer,cmd.getBytes("GB18030"));
send(writer,"flush_filter".getBytes());//flush_filter.pike->�رո�conn����
socket.shutdownOutput();

data=read(reader);
try{
if(writer!=null) writer.close();
if(reader!=null) reader.close();
if(socket!=null) socket.close();
}catch(Exception e)     
{
//logger.error("[uid:"+uid+"] [cmd:" + cmd+ "] [" +  (System.currentTimeMillis()-startTime)+"ms]",e);
} 
//logger.info("[uid:"+uid+"] [cmd:" + cmd+ "] [" +  (System.currentTimeMillis()-startTime)+"ms]");
//response.setContentType("text/vnd.wap.wml;charset=UTF-8");
//response.setContentType("text/html; charset=gb2312");
response.setContentType("text/html; charset=UTF-8");
%><%=data%>
