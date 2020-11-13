import logging
import threading

import RPi.GPIO as GPIO

from energymonitor.services.dispatcher import pubsub


class PressEvent:
    pass


class WakeupEvent:
    pass


class InactivityEvent:
    pass


class Button:
    """
    Service responsible for handling interactions with push button.
    """

    active = True
    inactivity_watcher_thread = None

    def __init__(self) -> None:
        self.logger = logging.getLogger(self.__class__.__name__)

        GPIO.setmode(GPIO.BCM)
        GPIO.setup(27, GPIO.IN, pull_up_down=GPIO.PUD_UP)  # GPIO27 pin 13

        def when_pressed(channel):
            if self.active:
                pubsub.publish(PressEvent())
            else:
                self.active = True
                pubsub.publish(WakeupEvent())
            self.reset_inactivity_watcher(30)
        GPIO.add_event_detect(27, GPIO.FALLING, callback=when_pressed, bouncetime=100)

        self.reset_inactivity_watcher(30)
        self.logger.debug('Initialized')

    def reset_inactivity_watcher(self, duration):
        def publish_inactivity_event():
            self.active = False
            pubsub.publish(InactivityEvent())
        if self.inactivity_watcher_thread:
            self.inactivity_watcher_thread.cancel()
        self.inactivity_watcher_thread = threading.Timer(duration, publish_inactivity_event)
        self.inactivity_watcher_thread.start()
