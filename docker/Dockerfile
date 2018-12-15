FROM ubuntu:16.04

RUN apt-get update
RUN apt-get install -y curl lib32gcc1 xz-utils

WORKDIR /opt/steamcmd
RUN curl -sqL "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" | tar zxvf -
RUN ./steamcmd.sh +quit # self update
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 90 validate +quit || :
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 70 validate +quit || :
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 10 validate +quit || :
RUN ./steamcmd.sh +login anonymous +force_install_dir /opt/hlds +app_update 90 validate +quit

ENV METAMOD_P_URL https://github.com/jkivilin/metamod-p/releases/download/1.21p38_beta1/metamod_i686_linux.win32.tar.xz
WORKDIR /opt/hlds/cstrike/addons/metamod
RUN curl -sSL $METAMOD_P_URL | tar xvfJ -
ADD plugins.ini .

WORKDIR /opt/hlds
RUN sed -i -- 's/gamedll_linux "dlls\/cs\.so"/gamedll_linux "addons\/metamod\/metamod.so"/g' cstrike/liblist.gam
RUN echo "10" > steam_appid.txt
CMD ["./hlds_run", "-game", "cstrike", "+maxplayers 4", "+map de_dust2", "-nomaster", "+log on", "+mp_logdetail 3"]
