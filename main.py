import re
from shlex import split
from subprocess import Popen, PIPE, STDOUT
from influxdb import InfluxDBClient
from datetime import datetime
from influxdb_client import InfluxDBClient, Point, WritePrecision
from influxdb_client.client.write_api import SYNCHRONOUS

def build_bazel():
    try:
        status_success = False
        build_command = "bazel build --profile=/tmp/prof //testapp:testapp"
        process = Popen(split(build_command), stdout=PIPE, stderr=STDOUT, encoding='utf8')
        while True:
            output = process.stdout.readline()
            if output == '' and process.poll() is not None:
                break
            if output:
                regex = 'Build completed successfully'
                if re.search(regex, output):
                    status_success = True
                print(output.strip())
        return status_success
    except KeyboardInterrupt:
        exit()
    except Exception as ex:
        print("Encountered an error : ", ex)


def analyze_bazel():
    try:
        time = 0.0
        build_command = "bazel analyze-profile /tmp/prof"
        process = Popen(split(build_command), stdout=PIPE, stderr=STDOUT, encoding='utf8')
        while True:
            output = process.stdout.readline()
            if output == '' and process.poll() is not None:
                break
            if output:
                regex_total = '(Total\\srun\\stime)(\\s+)([+-]?([0-9]*[.])?[0-9]+)'
                get_total = re.findall(regex_total, output)
                if get_total:
                    time = float(get_total[0][2])
                print(output.strip())
        return time
    except KeyboardInterrupt:
        exit()
    except Exception as ex:
        print("Encountered an error : ", ex)


def send_to_influxdb(time):
    token = "R5ONurHkUAi9TtiTg_YMtkoDuik5l3PXLgLLGSuHMCyJyI1VvejyvoCAO7MrDvdOTVSxKeZrZxN2FddBzn6DuQ=="
    org = "example"
    bucket = "bazel"

    client = InfluxDBClient(url="http://127.0.0.1:8086", token=token)

    write_api = client.write_api(write_options=SYNCHRONOUS)

    point = Point("mem")\
        .tag("host", "build")\
        .field("total run time", time)\
        .time(datetime.utcnow(), WritePrecision.NS)

    write_api.write(bucket, org, point)


if __name__ == '__main__':
    print("Building Bazel ...")
    if build_bazel():
        print("\nAnalyze profile ...")
        run_time = analyze_bazel()
        if run_time > 0:
            print("\nTotal run time is" + str(run_time) + "s")
            send_to_influxdb(run_time)

