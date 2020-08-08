import logging
from concurrent.futures import ThreadPoolExecutor
from queue import Queue
from threading import Thread
from typing import Callable


class Dispatcher(Thread):
    """
    Class responsible for dispatching messages to subscribed handlers.
    """

    def __init__(self):
        super().__init__(name=self.__class__.__name__)
        self.logger = logging.getLogger(self.__class__.__name__)
        self.queue = Queue()
        self.handlers = []
        self.executor = ThreadPoolExecutor(max_workers=2)

    def subscribe(self, handler: Callable):
        self.handlers.append(handler)

    def publish(self, message):
        self.queue.put(message)

    def run(self):
        while True:
            message = self.queue.get()
            for handler in self.handlers:
                self.executor.submit(handler, message)
            self.queue.task_done()


pubsub = Dispatcher()
pubsub.start()
