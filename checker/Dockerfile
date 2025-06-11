FROM python:3.12-slim-bookworm


RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y ffmpeg

# add checker user
RUN useradd -ms /bin/bash -u 1000 checker

WORKDIR /checker

RUN chmod -R 777 /checker
RUN chmod -R 777 .


COPY ./src .
RUN pwd
RUN ls
RUN chmod -R 777 videos

COPY requirements.txt .

USER checker
RUN python3 -m pip install -r requirements.txt





ENTRYPOINT [ "/home/checker/.local/bin/gunicorn", "-c", "gunicorn.conf.py", "checker:app" ]