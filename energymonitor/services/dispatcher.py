import logging
from concurrent.futures import ThreadPoolExecutor
from queue import Queue
from threading import Thread, Lock
from typing import Callable


class Dispatcher(Thread):
    """
    Class responsible for dispatching messages to subscribed handlers.
    """

    def __init__(self):
        super().__init__(name=self.__class__.__name__)
        self.logger = logging.getLogger(self.__class__.__name__)
        self.queue = Queue()
        self.subscription_lock = Lock()
        self.subscribers = []

    def subscribe(self, name: str, callback: Callable):
        with self.subscription_lock:
            executor = ThreadPoolExecutor(thread_name_prefix=f'{self.name}-{name}', max_workers=1)
            self.subscribers.append((name, executor, callback))
            self.logger.debug('Added subscriber %s', name)

    def publish(self, message):
        self.queue.put(message)

    def run(self):
        while True:
            message = self.queue.get()
            for name, executor, handler in self.subscribers:
                self.logger.debug('Publishing message %s to %s', message, name)
                executor.submit(handler, message)
            self.queue.task_done()


pubsub = Dispatcher()
pubsub.start()
