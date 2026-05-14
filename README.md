# Audio Sink Nexus

[English](./README.en.md) | [简体中文](./README.md) | [繁體中文](./README.zh_TW.md)

---

Audio Sink Nexus 是一款基于 Windows 10 AudioPlaybackConnection API 的音频接收端工具，可将手机、平板等支持蓝牙 A2DP Source 的设备音频无线传输到 Windows 电脑播放。

> 本项目基于 [ysc3839/AudioPlaybackConnector](https://github.com/ysc3839/AudioPlaybackConnector) 进行扩展和改进。

## 使用说明

1. **下载与安装**
    从[发布页面](https://github.com/littrick/litSinkNexus/releases)下载安装包或免安装版本，并按需安装或解压。


2. **启动软件**
    启动软件后，可在屏幕右下角查看已配对蓝牙设备的连接状态，任务栏区域会显示应用图标。


3. **配对设备**
    右键点击任务栏图标打开蓝牙列表，或直接在 Windows 蓝牙设置中，将需要连接的蓝牙 A2DP Source 设备与电脑完成配对。
    ![](doc_assets/zh/add.gif)


4. **连接设备**
    左键点击任务栏图标，打开已配对设备列表，再选择目标设备即可建立连接。
    ![](doc_assets/zh/connect.gif)


5. **播放音频**
    连接成功后，在已连接设备上播放音频，声音会通过无线传输到电脑端播放。


6. **断开连接**
    如需断开连接，左键点击任务栏图标，选择已连接设备后执行断开即可。