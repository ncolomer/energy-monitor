import logging
import threading

import gpiozero

from energymonitor.services.dispatcher import pubsub


class PressEvent:
    pass


class InactivityEvent:
    pass


class Button:
    """
    Service responsible for handling interactions with push button.
    """

    inactivity_watcher_thread = None

    def __init__(self) -> None:
        self.logger = logging.getLogger(self.__class__.__name__)

        button = gpiozero.Button(
            pin=27,  # GPIO27 pin 13
            pull_up=True,
            bounce_time=0.050,  # in seconds
        )

        def when_pressed():
            pubsub.publish(PressEvent())
            self.reset_inactivity_watcher(30)
        button.when_pressed = when_pressed

        self.reset_inactivity_watcher(2 * 60)
        self.logger.debug('Initialized')

    def reset_inactivity_watcher(self, duration):
        def publish_inactivity_event():
            pubsub.publish(InactivityEvent())
        if self.inactivity_watcher_thread:
            self.inactivity_watcher_thread.cancel()
        self.inactivity_watcher_thread = threading.Timer(duration, publish_inactivity_event)
        self.inactivity_watcher_thread.start()
