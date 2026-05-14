# Audio Sink Nexus

[English](./README.en.md) | [简体中文](./README.md) | [繁體中文](./README.zh_TW.md)

---

Audio Sink Nexus 是一款基於 Windows 10 AudioPlaybackConnection API 的音訊接收工具，可將手機、平板等支援藍牙 A2DP Source 的裝置音訊無線傳輸到 Windows 電腦播放。

> 本專案基於 [ysc3839/AudioPlaybackConnector](https://github.com/ysc3839/AudioPlaybackConnector) 進行擴充與改進。

## 使用說明

1. **下載與安裝**
    從[發佈頁面](https://github.com/littrick/litSinkNexus/releases)下載安裝版或免安裝版本，並依需求安裝或解壓縮。


2. **啟動軟體**
    啟動後，可在畫面右下角查看已配對藍牙裝置的連線狀態，工作列區域也會顯示應用程式圖示。


3. **配對裝置**
    右鍵點擊工作列圖示開啟藍牙清單，或直接在 Windows 藍牙設定中，將需要連線的藍牙 A2DP Source 裝置與電腦完成配對。
    ![](doc_assets/zh/add.gif)


4. **連線裝置**
    左鍵點擊工作列圖示，開啟已配對裝置清單後，選擇目標裝置即可建立連線。
    ![](doc_assets/zh/connect.gif)


5. **播放音訊**
    連線成功後，在已連線裝置上播放音訊，聲音便會無線傳輸到電腦端播放。


6. **中斷連線**
    如需中斷連線，左鍵點擊工作列圖示，選擇已連線裝置後執行中斷即可。