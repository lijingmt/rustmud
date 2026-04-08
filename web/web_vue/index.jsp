<%@ page language="java" contentType="text/html;charset=UTF-8" pageEncoding="UTF-8"%>
<%
    // 直接转发到 index.html
    RequestDispatcher dispatcher = request.getRequestDispatcher("index.html");
    dispatcher.forward(request, response);
%>
